use core::fmt::Write;

use heapless::String;

use super::Frame;
use crate::{collection, datastore, fcs::out::FCS, types::measurement::unit::Feet};

pub struct Telemetry;

macro_rules! next_row {
    ($frame:ident, $buf:ident, $row:ident) => {
        $frame[$row][..$buf.len()].copy_from_slice($buf.as_bytes());
        $buf.clear();
        $row += 1;
    };
}

impl Telemetry {
    pub fn draw<'a, const W: usize, const H: usize>(
        &self,
        frame: &'a mut Frame<W, H>,
    ) -> &'a Frame<W, H> {
        let mut buf: String<W> = String::new();
        let collection = collection::Collector::new(datastore::acquire()).collect();
        let mut row = 0;

        let att = collection.imu.attitude;
        write!(buf, "ATT {:3} {:3} {:3}", att.roll as i16, att.pitch as i16, att.yaw as i16).ok();
        next_row!(frame, buf, row);

        let pos = collection.ins.position;
        write!(buf, "POS {} {}", pos.latitude, pos.longitude).ok();
        next_row!(frame, buf, row);

        write!(buf, "ALT {}FT", pos.altitude.0.u(Feet).t(|v| v as isize)).ok();
        next_row!(frame, buf, row);

        let axes = collection.control.axes;
        write!(buf, "RC  {:4}T {:4} {:4} {:4}", axes.throttle, axes.roll, axes.pitch, axes.yaw)
            .ok();
        next_row!(frame, buf, row);

        write!(buf, "BAT {}V", collection.voltage.0).ok();
        next_row!(frame, buf, row);

        match collection.fcs {
            FCS::FixedWing(fixed_wing) => {
                write!(buf, "ENG {:4}", fixed_wing.engines[0] / 16).ok();
                next_row!(frame, buf, row);

                write!(buf, "SVO").ok();
                for (_, v) in fixed_wing.control_surface.iter() {
                    write!(buf, " {:4}", v / 16).ok();
                }
            }
        }
        next_row!(frame, buf, row);
        let _ = row;

        frame
    }
}
