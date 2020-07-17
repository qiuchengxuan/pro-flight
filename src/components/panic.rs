use core::fmt::{Arguments, Display, Result, Write};

use crate::logger;
use crate::sys::fs::OpenOptions;

const VALID_LOG: u32 = 0xCAFEFEED;
const INVALID_LOG: u32 = 0xDEADBEEF;

pub struct PanicLogger {
    valid: u32,
    size: usize,
    content: [u8; 1024],
}

impl PanicLogger {
    pub fn invalidate(&mut self) {
        self.valid = INVALID_LOG;
        self.size = 0;
    }

    pub fn is_valid(&self) -> bool {
        self.valid == VALID_LOG
    }
}

impl Display for PanicLogger {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", unsafe { core::str::from_utf8_unchecked(&self.content[..self.size]) })
    }
}

impl Write for PanicLogger {
    fn write_char(&mut self, c: char) -> Result {
        if self.size < self.content.len() {
            self.content[self.size] = c as u8;
            self.size += 1;
        }
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result {
        if self.size < self.content.len() {
            let bytes = s.as_bytes();
            let buffer = &mut self.content[self.size..];
            let size = core::cmp::min(bytes.len(), buffer.len());
            buffer[..size].copy_from_slice(&bytes[..size]);
            self.size += size;
        }
        Ok(())
    }
}

pub unsafe fn log_panic(args: Arguments, panic_logger: &mut PanicLogger) {
    *panic_logger = PanicLogger { valid: VALID_LOG, size: 0, content: [0u8; 1024] };
    writeln!(panic_logger, "{}", args).ok();
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open("sdcard://panic.log").ok() {
        writeln!(file, "{}", args).ok();
        for s in logger::reader() {
            file.write_str(s).ok();
        }
        file.close();
    }
}
