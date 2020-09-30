use core::fmt::{Arguments, Write};

use crate::components::logger;
use crate::sys::fs::OpenOptions;

pub unsafe fn log_panic(args: Arguments) {
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open("sdcard://panic.log").ok() {
        writeln!(file, "{}", args).ok();
        logger::get().write(&mut file).ok();
        file.close();
    }
}
