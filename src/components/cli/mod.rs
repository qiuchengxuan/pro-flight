mod config;
pub mod memory;

use alloc::vec::Vec;

use crate::alloc;
use crate::components::logger;
use crate::components::telemetry::TelemetryData;
use crate::datastructures::data_source::StaticData;
use crate::drivers::serial::Readline;
use crate::sys::timer::{get_jiffies, SysTimer};

pub struct CLI<T> {
    vec: Vec<u8>,
    timer: SysTimer,
    telemetry: T,
}

impl<T: StaticData<TelemetryData>> CLI<T> {
    pub fn new(telemetry: T) -> Self {
        CLI { vec: Vec::with_capacity(80), timer: SysTimer::new(), telemetry }
    }

    pub fn interact<S, E>(&mut self, serial: &mut S, mut extra: E) -> core::fmt::Result
    where
        E: FnMut(&str, &mut S) -> bool,
        S: core::fmt::Write + Readline,
    {
        let line = match serial.readline(&mut self.vec) {
            Some(line) => unsafe { core::str::from_utf8_unchecked(line) },
            None => return Ok(()),
        };
        if let Some(first_word) = line.split(' ').next() {
            match first_word {
                "logread" => write!(serial, "{}", logger::get()),
                "uptime" => write!(serial, "{:?}", get_jiffies()),
                "read" | "readx" => memory::read(line, serial),
                "dump" => memory::dump(line, serial),
                "write" => memory::write(line, serial, &mut self.timer),
                "set" => config::set(serial, line),
                "show" => config::show(serial),
                "save" => config::save(),
                "telemetry" => writeln!(serial, "{}", self.telemetry.read()),
                "" => Ok(()),
                _ => {
                    if extra(line, serial) {
                        Ok(())
                    } else {
                        writeln!(serial, "unknown input: {:?}", line)
                    }
                }
            }?
        }
        write!(serial, "# ")?;
        self.vec.truncate(0);
        Ok(())
    }
}
