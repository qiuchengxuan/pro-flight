use core::fmt::Write;

use heapless::String;

use super::Frame;
use crate::{collection, datastore, fcs::out::Configuration};

pub struct Telemetry;

macro_rules! next_row {
    ($frame:ident, $buf:ident, $row:ident, $height:ident) => {
        $frame[$row][..$buf.len()].copy_from_slice($buf.as_bytes());
        $buf.clear();
        $row += 1;
        if $row >= $height {
            return $frame;
        }
    };
}

fn percentage<T: Into<i32>>(value: T, max: T) -> i8 {
    (value.into() * 100 / max.into()) as i8
}

impl Telemetry {
    pub fn draw<'a, const W: usize, const H: usize>(
        &self,
        frame: &'a mut Frame<W, H>,
    ) -> &'a Frame<W, H> {
        let mut buf: String<W> = String::new();
        let collection = collection::Collector::new(datastore::acquire()).collect();
        let mut row = 0;

        frame.iter_mut().for_each(|line| {
            for ch in line.as_mut() {
                *ch = match *ch {
                    b' ' | 0 => 0,
                    _ => b' ',
                }
            }
        });

        let pos = collection.ins.position;
        let (lat, lon) = (pos.latitude.into::<'o'>(), pos.longitude.into::<'o'>());
        write!(buf, "POS {} {}", lat, lon).ok();
        next_row!(frame, buf, row, H);

        let att = collection.imu.attitude;
        write!(buf, "ATT {:4} {:4} {:4}", att.roll as i16, att.pitch as i16, att.yaw as i16).ok();
        next_row!(frame, buf, row, H);

        let axes = collection.control.axes;
        let throttle = percentage(axes.throttle, u16::MAX);
        let roll = percentage(axes.roll, i16::MAX);
        let pitch = percentage(axes.pitch, i16::MAX);
        let yaw = percentage(axes.yaw, i16::MAX);
        write!(buf, "RC  {:3}T {:4} {:4} {:4}", throttle, roll, pitch, yaw).ok();
        next_row!(frame, buf, row, H);

        match collection.fcs.control {
            Configuration::FixedWing(fixed_wing) => {
                write!(buf, "ENG {:3}", percentage(fixed_wing.engines[0], u16::MAX)).ok();
                next_row!(frame, buf, row, H);

                write!(buf, "CTL").ok();
                for &(_, v) in fixed_wing.control_surface.iter() {
                    write!(buf, " {:4}", percentage(v, i16::MAX)).ok();
                }
            }
        }
        next_row!(frame, buf, row, H);

        write!(buf, "BAT {}v", collection.voltage.0).ok();
        next_row!(frame, buf, row, H);

        let _ = row;

        frame
    }
}

mod test {
    #[test]
    fn test_telemetry() {
        use std::str::from_utf8;

        use super::{Frame, Telemetry};

        let mut frame = Frame::<30, 6>::default();
        let telemetry = Telemetry;
        let actual = telemetry
            .draw(&mut frame)
            .iter()
            .map(|bytes| from_utf8(bytes).unwrap().trim_end_matches('\0'))
            .collect::<Vec<_>>();
        let expected = vec![
            "POS N00o00'000 E000o00'000",
            "ATT    0    0    0",
            "RC    0T    0    0    0",
            "ENG   0",
            "CTL",
            "BAT 0.0v",
        ];
        assert_eq!(expected, actual);
    }
}
