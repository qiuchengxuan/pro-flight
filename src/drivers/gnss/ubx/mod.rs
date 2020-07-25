pub mod message;
pub mod nav_pos_pvt;

use crate::alloc;
use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::Distance;

use message::{Message, UBX_HEADER};
use nav_pos_pvt::{FixType, NavPositionVelocityTime, CLASS, ID};

const NAV_PVT_SIZE: usize = core::mem::size_of::<Message<NavPositionVelocityTime>>();
const NAV_PVT_HEADER: [u8; 4] = [UBX_HEADER[0], UBX_HEADER[1], CLASS, ID];

pub struct UBXDecoder {
    buffer: [u8; NAV_PVT_SIZE],
    remain: usize,
    position: &'static SingularData<Position>,
    fix_type: FixType,
    running: bool,
}

impl UBXDecoder {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; NAV_PVT_SIZE],
            remain: 0,
            position: alloc::into_static(SingularData::default(), false).unwrap(),
            fix_type: FixType::NoFix,
            running: false,
        }
    }

    pub fn as_position_source(&self) -> impl DataSource<Position> {
        SingularDataSource::new(self.position)
    }

    fn handle_pvt_message(&mut self) {
        let pvt_message: &Message<NavPositionVelocityTime> =
            unsafe { core::mem::transmute(&self.buffer) };
        if pvt_message.calc_checksum() != pvt_message.checksum {
            warn!("Inconsistent UBX checksum");
            return;
        }
        let payload = &pvt_message.payload;
        if payload.fix_type == FixType::ThreeDemension {
            self.position.write(Position {
                latitude: payload.latitude.into(),
                longitude: payload.longitude.into(),
                altitude: Distance(payload.height_above_msl as isize / 10),
            });
        }

        if payload.fix_type != self.fix_type {
            let fix_type: &str = payload.fix_type.into();
            info!("GNSS fix-type changed to {}", fix_type);
            self.fix_type = payload.fix_type;
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool, _: usize) {
        if half {
            return;
        }
        let mut index = 0;
        if self.remain > NAV_PVT_SIZE - NAV_PVT_HEADER.len() {
            let mut header_index = NAV_PVT_SIZE - self.remain;
            while index < ring.len() {
                let byte = ring[index];
                index += 1;
                if byte == NAV_PVT_HEADER[header_index] {
                    header_index += 1;
                    if header_index == NAV_PVT_HEADER.len() {
                        break;
                    }
                } else {
                    header_index = 0;
                }
            }
        }
        if index == ring.len() {
            return;
        }
        if self.remain <= NAV_PVT_SIZE - NAV_PVT_HEADER.len() {
            let size = core::cmp::min(self.remain, ring.len() / 2);
            let offset = NAV_PVT_SIZE - self.remain;
            self.buffer[offset..offset + size].copy_from_slice(&ring[index..index + size]);
            self.remain -= size;
        }
        if self.remain == 0 {
            if !self.running {
                self.running = true;
                info!("GNSS UBX start working");
            }
            self.handle_pvt_message();
            self.remain = NAV_PVT_SIZE;
        }
    }
}
