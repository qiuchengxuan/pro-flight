pub mod message;
pub mod nav_pos_pvt;

use core::mem::{size_of, transmute};

use chrono::naive::NaiveDateTime;
use fixed_point::FixedPoint;

use crate::{
    datastructures::{
        coordinate::Position,
        measurement::{distance::Distance, unit, Course, Heading, VelocityVector},
    },
    protocol::{serial, serial::gnss::DataSource},
    sync::{cell::Cell, DataWriter},
    sys::time,
};

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

pub struct UBX<'a> {
    state: State,
    fixed: &'a Cell<bool>,
    position: &'a Cell<Position>,
    velocity: &'a Cell<VelocityVector<i32, unit::MMpS>>,
    heading: &'a Cell<Heading>,
    course: &'a Cell<Course>,
    buffer: [u8; MAX_MESSAGE_SIZE],
}

impl<'a> UBX<'a> {
    pub fn new(data_source: DataSource<'a>) -> Self {
        Self {
            state: State::WaitHeader0,
            fixed: data_source.fixed,
            position: data_source.position,
            velocity: data_source.velocity,
            heading: data_source.heading,
            course: data_source.course,
            buffer: [0u8; MAX_MESSAGE_SIZE],
        }
    }

    fn handle_pvt_message(&mut self) {
        let pvt_message: &Message<NavPositionVelocityTime> = unsafe { transmute(&self.buffer) };
        if !pvt_message.validate_checksum() {
            return;
        }

        let payload = &pvt_message.payload;

        match (payload.date(), payload.time()) {
            (Some(date), Some(time)) => {
                time::update(&NaiveDateTime::new(date, time)).ok();
            }
            _ => (),
        }

        self.fixed.write(match payload.fix_type {
            UBXFixType::TwoDemension | UBXFixType::ThreeDemension => true,
            _ => false,
        });

        if payload.fix_type == UBXFixType::ThreeDemension {
            self.position.write(Position {
                latitude: payload.latitude.into(),
                longitude: payload.longitude.into(),
                altitude: Distance::new(payload.height_above_msl / 10, unit::CentiMeter),
            });
            let (x, y, z) = (payload.velocity_east, payload.velocity_north, -payload.velocity_down);
            self.velocity.write(VelocityVector::new(x, y, z, unit::MMpS));

            let course = payload.heading_of_motion;
            let course = if course > 0 { course } else { 360_00000 + course } as i32;
            let course_valid = payload.ground_speed / 1000 > 0;
            if course_valid {
                self.course.write(FixedPoint(course / 10000));
            }

            let heading = payload.heading_of_vehicle;
            let heading = if heading > 0 { heading } else { 360_00000 + heading } as i32;
            if payload.flags1.heading_of_vehicle_valid() {
                self.heading.write(FixedPoint(heading / 10000));
            }
        }
    }
}

impl<'a> serial::Receiver for UBX<'a> {
    fn receive_size(&self) -> usize {
        MAX_MESSAGE_SIZE
    }

    fn receive(&mut self, mut bytes: &[u8]) {
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

    fn reset(&mut self) {
        self.state = State::WaitHeader0;
    }
}

mod test {
    #[test]
    fn test_message() {
        use hex_literal::hex;

        use crate::{
            components::flight_data_hub::FlightDataHUB,
            protocol::serial::{
                gnss::{ubx::message::Message, DataSource},
                Receiver,
            },
            sync::DataReader,
        };

        use super::{NavPositionVelocityTime, UBX};

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
        let hub = FlightDataHUB::default();
        let mut reader = hub.reader();
        let data_source = DataSource {
            fixed: &hub.gnss_fixed,
            position: &hub.gnss_position,
            velocity: &hub.gnss_velocity,
            heading: &hub.gnss_heading,
            course: &hub.gnss_course,
        };
        let mut ubx = UBX::new(data_source);
        let position = &mut reader.gnss_position;
        ubx.receive(&message[0..64]);
        assert_eq!(position.get().is_none(), true);
        ubx.receive(&message[64..message.len()]);
        assert_eq!(position.get().is_some(), true);
    }
}
