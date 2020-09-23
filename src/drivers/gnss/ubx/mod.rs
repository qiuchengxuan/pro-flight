pub mod message;
pub mod nav_pos_pvt;

use alloc::rc::Rc;
use core::mem::size_of;
use core::mem::transmute;

use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::u16_source::{U16Data, U16DataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, MilliMeter};
use crate::datastructures::measurement::VelocityVector;
use crate::datastructures::measurement::{Course, Heading, HeadingOrCourse};
use crate::datastructures::GNSSFixed;

use message::{Message, CHECKSUM_SIZE, PAYLOAD_OFFSET, UBX_HEADER0, UBX_HEADER1};
use nav_pos_pvt::{FixType as UBXFixType, NavPositionVelocityTime};

const MAX_MESSAGE_SIZE: usize = size_of::<Message<NavPositionVelocityTime>>();

#[derive(Copy, Clone)]
pub enum State {
    WaitHeader0,
    WaitHeader1,
    WaitClass,
    WaitId,
    WaitLength1,
    WaitLength0,
    Skip(usize),
    Remain(usize),
}

pub struct UBXDecoder {
    state: State,
    fixed: Rc<U16Data<GNSSFixed>>,
    position: Rc<SingularData<Position>>,
    velocity: Rc<SingularData<VelocityVector<i32, MilliMeter>>>,
    heading: Rc<SingularData<HeadingOrCourse>>,
    course: Rc<SingularData<Course>>,
    buffer: [u8; MAX_MESSAGE_SIZE],
}

impl UBXDecoder {
    pub fn new() -> Self {
        Self {
            state: State::WaitHeader0,
            fixed: Rc::new(U16Data::default()),
            position: Rc::new(SingularData::default()),
            velocity: Rc::new(SingularData::default()),
            heading: Rc::new(SingularData::default()),
            course: Rc::new(SingularData::default()),
            buffer: [0u8; MAX_MESSAGE_SIZE],
        }
    }

    pub fn fixed(&self) -> U16DataSource<GNSSFixed> {
        U16DataSource::new(&self.fixed)
    }

    pub fn position(&self) -> SingularDataSource<Position> {
        SingularDataSource::new(&self.position)
    }

    pub fn velocity(&self) -> SingularDataSource<VelocityVector<i32, MilliMeter>> {
        SingularDataSource::new(&self.velocity)
    }

    pub fn course(&self) -> SingularDataSource<Course> {
        SingularDataSource::new(&self.course)
    }

    pub fn heading(&self) -> SingularDataSource<HeadingOrCourse> {
        SingularDataSource::new(&self.heading)
    }

    fn handle_pvt_message(&mut self) {
        let pvt_message: &Message<NavPositionVelocityTime> = unsafe { transmute(&self.buffer) };
        if !pvt_message.validate_checksum() {
            return;
        }

        let payload = &pvt_message.payload;

        self.fixed.write(match payload.fix_type {
            UBXFixType::TwoDemension | UBXFixType::ThreeDemension => GNSSFixed(true),
            _ => GNSSFixed(false),
        });

        if payload.fix_type == UBXFixType::ThreeDemension {
            self.position.write(Position {
                latitude: payload.latitude.into(),
                longitude: payload.longitude.into(),
                altitude: Distance::new(payload.height_above_msl / 10, CentiMeter),
            });
            let (x, y, z) = (payload.velocity_east, payload.velocity_north, -payload.velocity_down);
            self.velocity.write(VelocityVector::new(x, y, z, MilliMeter));

            let course = payload.heading_of_motion;
            let course = if course > 0 { course } else { 360_00000 + course } as i32;
            let course = Course::new(course / 10, 4);
            let course_valid = payload.ground_speed / 1000 > 0;
            if course_valid {
                self.course.write(course);
            }

            let heading = payload.heading_of_vehicle;
            let heading = if heading > 0 { heading } else { 360_00000 + heading } as i32;
            let heading = Heading::new(heading / 10, 4);
            if payload.flags1.heading_of_vehicle_valid() {
                self.heading.write(HeadingOrCourse::Heading(heading));
            } else if course_valid {
                self.heading.write(HeadingOrCourse::Course(course));
            }
        }
    }

    pub fn handle(&mut self, mut bytes: &[u8]) {
        let mut message: &mut Message<()> = unsafe { transmute(&mut self.buffer) };
        while bytes.len() > 0 {
            match (self.state, bytes[0]) {
                (State::WaitHeader0, UBX_HEADER0) => {
                    self.state = State::WaitHeader1;
                }
                (State::WaitHeader1, UBX_HEADER1) => {
                    self.state = State::WaitClass;
                }
                (State::WaitClass, class) => {
                    message.class = class;
                    self.state = State::WaitId;
                }
                (State::WaitId, id) => {
                    message.id = id;
                    self.state = State::WaitLength0;
                }
                (State::WaitLength0, value) => {
                    message.length = value as u16;
                    self.state = State::WaitLength1;
                }
                (State::WaitLength1, value) => {
                    let length = u16::from_le_bytes([message.length as u8, value]);
                    message.length = u16::to_le(length);
                    let length = length as usize + CHECKSUM_SIZE;
                    self.state = match message.payload_type() {
                        Some(_) => State::Remain(length),
                        None => State::Skip(length),
                    }
                }
                (State::Skip(size), _) => {
                    if bytes.len() < size {
                        self.state = State::Skip(size - bytes.len());
                        return;
                    }
                    bytes = &bytes[size..];
                    self.state = State::WaitHeader0;
                    continue;
                }
                (State::Remain(size), _) => {
                    let offset = PAYLOAD_OFFSET + (message.length() + CHECKSUM_SIZE) - size;
                    let buffer = &mut self.buffer[offset..];
                    if bytes.len() < size {
                        buffer[..bytes.len()].copy_from_slice(bytes);
                        self.state = State::Remain(size - bytes.len());
                        return;
                    }
                    buffer[..size].copy_from_slice(&bytes[..size]);
                    self.handle_pvt_message();
                    bytes = &bytes[size..];
                    self.state = State::WaitHeader0;
                    continue;
                }
                _ => {
                    self.state = State::WaitHeader0;
                }
            }
            bytes = &bytes[1..];
        }
    }
}

mod test {
    #[test]
    fn test_message() {
        use crate::datastructures::data_source::OptionData;
        use crate::drivers::gnss::ubx::message::Message;

        use super::{NavPositionVelocityTime, UBXDecoder};

        assert_eq!(core::mem::size_of::<Message<NavPositionVelocityTime>>(), 100);

        let message = hex!(
            "00 00
             B5 62 01 07 5C 00
             00 00 00 00 E0 07 0A 15 16 0D 0A 04 01 00 00 00
             01 00 00 00 03 0C E0 0B 86 BE 2F FF AD 1F 21 04
             E0 F2 09 00 A0 56 09 00 01 00 00 00 01 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00
             D6 73"
        );
        let mut decoder = UBXDecoder::new();
        let mut position = decoder.position();
        decoder.handle(&message[0..64]);
        assert_eq!(position.read().is_none(), true);
        decoder.handle(&message[64..message.len()]);
        assert_eq!(position.read().is_some(), true);
    }
}
