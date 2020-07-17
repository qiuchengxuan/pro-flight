use core::time::Duration;

use arrayvec::ArrayVec;
use embedded_hal::serial::{Read, Write};
use embedded_hal::timer::CountDown;

use crate::components::cmdlet;
use crate::components::console::{self, Console};
use crate::config;
use crate::config::yaml::ToYAML;
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
        if line.len() > 0 {
            if extra(line, serial) {
                return;
            }
            if line.starts_with("logread") {
                for s in logger::reader() {
                    console!(serial, "{}", s);
                }
            } else if line.starts_with("read") {
                cmdlet::read(line, serial);
            } else if line.starts_with("dump ") {
                cmdlet::dump(line, serial);
            } else if line.starts_with("write ") {
                cmdlet::write(line, serial, &mut self.count_down);
            } else if line.starts_with("show config") {
                config::get().write_to(0, &mut Console(serial)).ok();
            } else {
                console!(serial, "unknown input\n");
            }
        }
        console!(serial, "# ");
        self.vec.clear();
    }
}
