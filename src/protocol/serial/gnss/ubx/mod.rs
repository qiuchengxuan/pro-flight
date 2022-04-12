pub mod message;
pub mod nav_pos_pvt;

use core::mem::{size_of, transmute};

use chrono::naive::NaiveDateTime;
use fixed_point::FixedPoint;

use super::out::{Fixed, GNSS};
use crate::types::{
    coordinate::Position,
    measurement::{unit, Altitude, Course, Distance, Heading, Velocity, VelocityVector, ENU},
};

use message::{Message, CHECKSUM_SIZE, PAYLOAD_OFFSET, UBX_HEADER0, UBX_HEADER1};
use nav_pos_pvt::{FixType as UBXFixType, NavPositionVelocityTime};

const MAX_MESSAGE_SIZE: usize = size_of::<Message<NavPositionVelocityTime>>();
pub const CHUNK_SIZE: usize = MAX_MESSAGE_SIZE;

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

pub struct UBX {
    state: State,
    buffer: [u8; MAX_MESSAGE_SIZE],
}

impl UBX {
    pub fn new() -> Self {
        Self { state: State::WaitHeader0, buffer: [0u8; MAX_MESSAGE_SIZE] }
    }

    fn handle_pvt_message(&mut self) -> Option<GNSS> {
        let pvt_message: &Message<NavPositionVelocityTime> = unsafe { transmute(&self.buffer) };
        if !pvt_message.validate_checksum() {
            return None;
        }

        let payload = &pvt_message.payload;

        let datetime = match (payload.date(), payload.time()) {
            (Some(date), Some(time)) => Some(NaiveDateTime::new(date, time)),
            _ => None,
        };

        match payload.fix_type {
            UBXFixType::ThreeDemension | UBXFixType::GNSSPlusDeadReckoningCombined => {
                let altitude = Distance::new(payload.height_above_msl / 10, unit::CentiMeter);
                let position = Position {
                    latitude: payload.latitude.into(),
                    longitude: payload.longitude.into(),
                    altitude: Altitude(altitude),
                };

                let h = payload.heading_of_motion;
                let course =
                    Course(FixedPoint(if h > 0 { h } else { 360_00000 + h } as i32 / 10000));

                let h = payload.heading_of_vehicle;
                let heading = match payload.flags1.heading_of_vehicle_valid() {
                    true => Some(Heading(FixedPoint(
                        if h > 0 { h } else { 360_00000 + h } as i32 / 10000,
                    ))),
                    false => None,
                };
                let (x, y, z) =
                    (payload.velocity_east, payload.velocity_north, -payload.velocity_down);
                let ground_speed = Velocity::new(payload.ground_speed, unit::MMs);
                let velocity_vector = Some(VelocityVector::new(x, y, z, unit::MMs, ENU));
                let fixed = Fixed { position, course, heading, ground_speed, velocity_vector };
                Some(GNSS { datetime, fixed: Some(fixed) })
            }
            _ => Some(GNSS { datetime, fixed: None }),
        }
    }

    pub fn receive(&mut self, mut bytes: &[u8]) -> Option<GNSS> {
        let mut message: &mut Message<()> = unsafe { transmute(&mut self.buffer) };
        let mut gnss: Option<GNSS> = None;
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
                        return None;
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
                        return None;
                    }
                    buffer[..size].copy_from_slice(&bytes[..size]);
                    gnss = self.handle_pvt_message();
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
        gnss
    }

    pub fn reset(&mut self) {
        self.state = State::WaitHeader0;
    }
}

mod test {
    #[test]
    fn test_message() {
        use hex_literal::hex;

        use super::{message::Message, NavPositionVelocityTime, UBX};

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
        let mut ubx = UBX::new();
        let gnss = ubx.receive(&message[0..64]);
        assert_eq!(gnss.is_none(), true);
        let gnss = ubx.receive(&message[64..message.len()]);
        assert_eq!(gnss.is_some(), true);
    }
}
