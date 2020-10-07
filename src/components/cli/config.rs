use crate::config;
use crate::config::setter::{Setter, Value};
use crate::config::yaml::ToYAML;
use crate::sys::fs::OpenOptions;

pub fn show(writer: &mut impl core::fmt::Write) -> core::fmt::Result {
    config::get().write_to(0, writer)
}

pub fn set(writer: &mut impl core::fmt::Write, line: &str) -> core::fmt::Result {
    let mut split = line.split(' ');
    split.next();
    if let Some(path) = split.next() {
        let mut config = config::get().clone();
        match config.set(&mut path.split('.'), Value(split.next())) {
            Ok(()) => (),
            Err(e) => writeln!(writer, "{}", e)?,
        }
        config::replace(config);
    }
    Ok(())
}

pub fn save() -> core::fmt::Result {
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open("sdcard://config.yml").ok() {
        config::get().write_to(0, &mut file)?;
        file.close();
    }
    Ok(())
}
