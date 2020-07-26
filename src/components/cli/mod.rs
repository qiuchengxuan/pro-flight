mod config;
pub mod memory;

use arrayvec::ArrayVec;
use embedded_hal::serial::{Read, Write};

use crate::components::console;
use crate::logger;
use crate::sys::timer::SysTimer;

pub struct CLI {
    vec: ArrayVec<[u8; 80]>,
    timer: SysTimer,
}

impl CLI {
    pub fn new() -> Self {
        CLI { vec: ArrayVec::new(), timer: SysTimer::new() }
    }

    pub fn interact<RE, WE, S, E>(&mut self, serial: &mut S, mut extra: E)
    where
        E: FnMut(&str, &mut S) -> bool,
        S: Read<u8, Error = RE> + Write<u8, Error = WE>,
    {
        let line = match console::read_line(serial, &mut self.vec) {
            Some(line) => unsafe { core::str::from_utf8_unchecked(line) },
            None => return,
        };
        if let Some(first_word) = line.split(' ').next() {
            match first_word {
                "logread" => {
                    for s in logger::reader() {
                        console!(serial, "{}", s);
                    }
                }
                "read" | "readx" | "readf" => memory::read(line, serial),
                "dump" => memory::dump(line, serial),
                "write" => memory::write(line, serial, &mut self.timer),
                "set" => config::set(serial, line),
                "show" => config::show(serial),
                "save" => config::save(),
                "" => (),
                _ => {
                    if !extra(line, serial) {
                        console!(serial, "unknown input: {:?}\n", line);
                    }
                }
            }
        }
        console!(serial, "# ");
        self.vec.clear();
    }
}
