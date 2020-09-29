mod config;
pub mod memory;

use alloc::vec::Vec;
use core::fmt;

use git_version::git_version;

use crate::alloc;
use crate::components::logger;
use crate::components::telemetry::TelemetryData;
use crate::datastructures::data_source::StaticData;
use crate::drivers::serial::Readline;
use crate::sys::timer::SysTimer;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();

pub struct CLI<T> {
    vec: Vec<u8>,
    timer: SysTimer,
    telemetry: T,
    reboot: fn(),
    bootloader: fn(),
    free: fn() -> (usize, usize),
}

impl<T: StaticData<TelemetryData>> CLI<T> {
    pub fn new(telemetry: T, reboot: fn(), bootloader: fn(), free: fn() -> (usize, usize)) -> Self {
        CLI {
            vec: Vec::with_capacity(80),
            timer: SysTimer::new(),
            telemetry,
            reboot,
            bootloader,
            free,
        }
    }

    pub fn interact(&mut self, serial: &mut (impl Readline + fmt::Write)) -> fmt::Result {
        let line = match serial.readline(&mut self.vec) {
            Some(line) => unsafe { core::str::from_utf8_unchecked(line) },
            None => return Ok(()),
        };
        if let Some(first_word) = line.split(' ').next() {
            match first_word {
                "logread" => write!(serial, "{}", logger::get())?,
                "read" | "readx" => memory::read(line, serial)?,
                "dump" => memory::dump(line, serial)?,
                "write" => memory::write(line, serial, &mut self.timer)?,
                "set" => config::set(serial, line)?,
                "show" => config::show(serial)?,
                "save" => config::save()?,
                "telemetry" => writeln!(serial, "{}", self.telemetry.read())?,
                "version" => writeln!(serial, "{}-{}", VERSION, REVISION)?,
                "reboot" => (self.reboot)(),
                "bootloader" => (self.bootloader)(),
                "free" => {
                    let (used, free) = (self.free)();
                    writeln!(serial, "Used: {}, free: {}", used, free)?;
                }
                _ => (),
            }
        }
        write!(serial, "# ")?;
        self.vec.truncate(0);
        Ok(())
    }
}
