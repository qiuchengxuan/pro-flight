use alloc::vec::Vec;
use core::fmt::{Display, Result, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::sys::timer::get_jiffies;

#[derive(Copy, Clone, PartialEq)]
pub enum Level {
    Debug = 0,
    Info,
    Warning,
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO "),
            Self::Warning => write!(f, "WARN "),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

pub struct Logger {
    buffer: Vec<u8>,
    write_index: AtomicUsize,
    level: Level,
}

static mut LOGGER: Logger =
    Logger { buffer: Vec::new(), write_index: AtomicUsize::new(0), level: Level::Debug };

impl Logger {
    fn allocate(&self, len: usize) -> usize {
        let mut index = self.write_index.load(Ordering::Relaxed);
        loop {
            let new_index = index + len;
            let current = self.write_index.compare_and_swap(index, new_index, Ordering::Relaxed);
            if current == index {
                return index as usize;
            }
            index = current;
        }
    }
}

impl Write for Logger {
    fn write_char(&mut self, c: char) -> Result {
        if self.buffer.len() == 0 {
            return Ok(());
        }
        let index = self.allocate(1);
        let size = self.buffer.len();
        self.buffer[index % size] = c as u8;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result {
        let bytes = s.as_bytes();
        if self.buffer.len() <= bytes.len() {
            return Ok(());
        }
        let index = self.allocate(bytes.len()) % self.buffer.len();
        let buffer = &mut self.buffer;
        if index + bytes.len() < buffer.len() {
            buffer[index..index + bytes.len()].copy_from_slice(bytes);
        } else {
            let size = buffer.len() - index;
            buffer[index..index + size].copy_from_slice(&bytes[..size]);
            buffer[..bytes.len() - size].copy_from_slice(&bytes[size..]);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn __write_log(args: core::fmt::Arguments, level: Level) {
    let logger = unsafe { &mut LOGGER };
    if level < logger.level {
        return;
    }
    let jiffies = get_jiffies();
    let seconds = jiffies.as_secs() as u32;
    write!(logger, "[{:5}.{:03}] {} ", seconds, jiffies.subsec_millis(), level).ok();
    writeln!(logger, "{}", args).ok();
}

#[doc(hidden)]
pub fn __write_log_literal(message: &'static str, level: Level) {
    let logger = unsafe { &mut LOGGER };
    if level < logger.level {
        return;
    }
    let jiffies = get_jiffies();
    let seconds = jiffies.as_secs() as u32;
    write!(logger, "[{:5}.{:03}] {} ", seconds, jiffies.subsec_millis(), level).ok();
    writeln!(logger, "{}", message).ok();
}

#[doc(hidden)]
#[macro_export]
macro_rules! __format_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[macro_export]
macro_rules! log {
    ($level:expr, $message:expr) => ({
        let _ = __format_args!($message);
        $crate::logger::__write_log_literal($message, $level);
    });
    ($level:expr, $($arg:tt)+) => {
        $crate::logger::__write_log(__format_args!($($arg)+), $level);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        log!($crate::logger::Level::Debug, $($arg)+);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        log!($crate::logger::Level::Info, $($arg)+);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        log!($crate::logger::Level::Warning, $($arg)+);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        log!($crate::logger::Level::Error, $($arg)+);
    };
}

pub struct LogReader((usize, usize));

impl Iterator for LogReader {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        let (index, count) = self.0;
        let logger = unsafe { &LOGGER };
        if index <= logger.buffer.len() {
            if count == 0 {
                self.0 = (index, 1);
                let bytes: &[u8] = logger.buffer.as_ref();
                return Some(unsafe { core::str::from_utf8_unchecked(&bytes[..index]) });
            } else {
                return None;
            }
        }
        if count == 0 {
            self.0 = (index, 1);
            let bytes: &[u8] = logger.buffer.as_ref();
            let bytes = &bytes[index % logger.buffer.len()..];
            return unsafe { core::str::from_utf8_unchecked(bytes) }.splitn(1, '\n').next();
        } else if count == 1 {
            self.0 = (index, 2);
            let bytes: &[u8] = logger.buffer.as_ref();
            let bytes = &bytes[..index % logger.buffer.len()];
            return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
        }
        None
    }
}

pub fn reader() -> LogReader {
    let index = unsafe { &LOGGER }.write_index.load(Ordering::Relaxed);
    LogReader((index, 0))
}

pub fn init(level: Level) {
    unsafe { LOGGER = Logger { buffer: vec![0u8; 4096], write_index: AtomicUsize::new(0), level } }
}
