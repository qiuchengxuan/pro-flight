use core::fmt::{Debug, Write};

use crate::components::logger;
use crate::hal::io::Write as _;
use crate::sys::fs::OpenOptions;

pub fn write_panic_file<T: Debug>(any: T) {
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    match option.open("sdcard://panic.log") {
        Ok(mut file) => {
            write!(file, "{:?}", any).ok();
            for s in logger::reader() {
                file.write(s).ok();
            }
            file.close();
        }
        Err(_) => (),
    }
}
