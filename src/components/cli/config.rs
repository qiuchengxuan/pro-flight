use embedded_hal::serial;

use crate::components::console::Console;
use crate::config;
use crate::config::setter::Setter;
use crate::config::yaml::ToYAML;
use crate::sys::fs::OpenOptions;

pub fn show<WE, S: serial::Write<u8, Error = WE>>(serial: &mut S) {
    config::get().write_to(0, &mut Console(serial)).ok();
}

pub fn set<WE, S: serial::Write<u8, Error = WE>>(serial: &mut S, line: &str) {
    let mut split = line.split(' ');
    split.next();
    if let Some(path) = split.next() {
        let mut config = config::get().clone();
        match config.set(&mut path.split('.'), split.next()) {
            Ok(()) => (),
            Err(e) => console!(serial, "{}", e),
        }
        config::replace(&config);
    }
}

pub fn save() {
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open("sdcard://config.yml").ok() {
        config::get().write_to(0, &mut file).ok();
        file.close();
    }
}
