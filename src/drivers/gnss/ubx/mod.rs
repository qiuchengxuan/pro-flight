pub mod message;
pub mod nav_pos_pvt;

use alloc::rc::Rc;
use core::mem::transmute;

use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::gnss::FixType;
use crate::datastructures::measurement::distance::{CentiMeter, Distance};
use crate::datastructures::measurement::{Course, HeadingOrCourse, Velocity};

use message::{Message, CHECKSUM_SIZE, PAYLOAD_OFFSET, UBX_HEADER0, UBX_HEADER1};
use nav_pos_pvt::{FixType as UBXFixType, NavPositionVelocityTime};

const MAX_MESSAGE_SIZE: usize = core::mem::size_of::<Message<NavPositionVelocityTime>>();

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
    buffer: [u8; MAX_MESSAGE_SIZE],
    state: State,
    fix_type: Rc<SingularData<FixType>>,
    position: Rc<SingularData<Position>>,
    velocity: Rc<SingularData<[Velocity<i32>; 3]>>,
    heading: Rc<SingularData<HeadingOrCourse>>,
    course: Rc<SingularData<Course>>,
    normal: bool,
}

impl UBXDecoder {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; MAX_MESSAGE_SIZE],
            state: State::WaitHeader0,
            fix_type: Rc::new(SingularData::default()),
            position: Rc::new(SingularData::default()),
            velocity: Rc::new(SingularData::<[Velocity<i32>; 3]>::default()),
            course: Rc::new(SingularData::default()),
            heading: Rc::new(SingularData::default()),
            normal: true,
        }
    }

    pub fn fix_type(&self) -> impl DataSource<FixType> {
        SingularDataSource::new(&self.fix_type)
    }

    pub fn position(&self) -> impl DataSource<Position> {
        SingularDataSource::new(&self.position)
    }

    pub fn velocity(&self) -> impl DataSource<[Velocity<i32>; 3]> {
        SingularDataSource::new(&self.velocity)
    }

    pub fn course(&self) -> impl DataSource<Course> {
        SingularDataSource::new(&self.course)
    }

    pub fn heading(&self) -> impl DataSource<HeadingOrCourse> {
        SingularDataSource::new(&self.heading)
    }

    fn handle_pvt_message(&mut self) {
        let pvt_message: &Message<NavPositionVelocityTime> = unsafe { transmute(&self.buffer) };
        if !pvt_message.validate_checksum() && self.normal {
            warn!("UBX checksum invalid");
            return;
        }
        self.normal = true;

        let payload = &pvt_message.payload;

        let fix_type = match payload.fix_type {
            UBXFixType::TwoDemension => FixType::TwoDemension,
            UBXFixType::ThreeDemension => FixType::ThreeDemension,
            _ => FixType::NoFix,
        };
        self.fix_type.write(fix_type);

        if payload.fix_type == UBXFixType::ThreeDemension {
            self.position.write(Position {
                latitude: payload.latitude.into(),
                longitude: payload.longitude.into(),
                altitude: Distance::new(payload.height_above_msl / 10, CentiMeter),
            });
            self.velocity.write([
                Velocity(payload.velocity_north),
                Velocity(payload.velocity_east),
                Velocity(payload.velocity_down),
            ]);
            let course = payload.heading_of_motion / 10;
            self.course.write(if course > 0 { course } else { 360 + course } as u16);
            let hov = payload.heading_of_vehicle / 10;
            let heading = if hov > 0 { hov } else { 360 + hov } as u16;
            let heading_or_course = if payload.flags1.heading_of_vehicle_valid() {
                HeadingOrCourse::Heading(heading)
            } else {
                HeadingOrCourse::Course(heading)
            };
            self.heading.write(heading_or_course);
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool) {
        if half {
            return;
        }
        let mut index = 0;
        let mut message: &mut Message<()> = unsafe { transmute(&mut self.buffer) };
        while index < ring.len() {
            match (self.state, ring[index]) {
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
                    message.length = u16::from_le_bytes([message.length as u8, value]);
                    let length = message.length as usize + CHECKSUM_SIZE;
                    match message.payload_type() {
                        Some(_) => self.state = State::Remain(length),
                        None => self.state = State::Skip(length),
                    }
                }
                (State::Skip(size), _) => {
                    if ring.len() <= size {
                        self.state = State::Skip(size - ring.len());
                        return;
                    } else {
                        index = size;
                    }
                }
                (State::Remain(remain), _) => {
                    let offset = (message.length as usize + CHECKSUM_SIZE) - remain;
                    let payload = &mut self.buffer[PAYLOAD_OFFSET + offset..];
                    let size = ring.len() - index;
                    if size < remain {
                        payload[..size].copy_from_slice(&ring[index..]);
                        self.state = State::Remain(remain - (ring.len() - index));
                        return;
                    }
                    payload[..remain].copy_from_slice(&ring[index..index + remain]);
                    self.handle_pvt_message();
                    self.state = State::WaitHeader0;
                }
                _ => {
                    self.state = State::WaitHeader0;
                }
            }
            index += 1;
        }
    }
}

mod test {
    #[test]
    fn test_message() {
        use crate::datastructures::data_source::DataSource;
        use crate::drivers::gnss::ubx::message::Message;

        use super::NavPositionVelocityTime;
        use super::UBXDecoder;

        assert_eq!(core::mem::size_of::<Message<NavPositionVelocityTime>>(), 100);

        let message = hex!(
            "00 00
             B5 62 01 07 5C 00 00 00 00 00 E0 07 0A 15 16 0D
             0A 04 01 00 00 00 01 00 00 00 03 0C E0 0B 86 BE
             2F FF AD 1F 21 20 E0 F2 09 00 A0 56 09 00 01 00
             00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 1F F2 00 00"
        );
        let mut decoder = UBXDecoder::new();
        let mut position = decoder.position();
        decoder.handle(&message[0..64], false);
        decoder.handle(&message[64..message.len()], false);
        assert_eq!(position.read_last().is_some(), true);
    }
}
