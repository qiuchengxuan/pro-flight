mod config;
pub mod memory;

use alloc::vec::Vec;

use embedded_hal::serial::{Read, Write};

use crate::alloc;
use crate::components::{console, logger};
use crate::sys::timer::{get_jiffies, SysTimer};

pub struct CLI {
    vec: Vec<u8>,
    timer: SysTimer,
}

impl CLI {
    pub fn new() -> Self {
        CLI { vec: Vec::with_capacity(80), timer: SysTimer::new() }
    }

    pub fn interact<RE, WE, S, E>(&mut self, serial: &mut S, mut extra: E)
    where
        E: FnMut(&str) -> bool,
        S: Read<u8, Error = RE> + Write<u8, Error = WE>,
    {
        let line = match console::readline(serial, &mut self.vec) {
            Some(line) => unsafe { core::str::from_utf8_unchecked(line) },
            None => return,
        };
        if let Some(first_word) = line.split(' ').next() {
            match first_word {
                "logread" => print!("{}", logger::read()),
                "uptime" => println!("{:?}", get_jiffies()),
                "read" | "readx" | "readf" => memory::read(line),
                "dump" => memory::dump(line),
                "write" => memory::write(line, &mut self.timer),
                "set" => config::set(line),
                "show" => config::show(),
                "save" => config::save(),
                "" => (),
                _ => {
                    if !extra(line) {
                        println!("unknown input: {:?}", line);
                    }
                }
            }
        }
        print!("# ");
        self.vec.truncate(0);
    }
}
