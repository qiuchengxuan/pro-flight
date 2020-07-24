mod config;
pub mod memory;

use core::time::Duration;

use arrayvec::ArrayVec;
use embedded_hal::serial::{Read, Write};
use embedded_hal::timer::CountDown;

use crate::components::console;
use crate::logger;

pub struct CLI<C> {
    vec: ArrayVec<[u8; 80]>,
    count_down: C,
}

impl<C> CLI<C>
where
    C: CountDown<Time = Duration>,
{
    pub fn new(count_down: C) -> Self {
        CLI { vec: ArrayVec::new(), count_down }
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
                "write" => memory::write(line, serial, &mut self.count_down),
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
