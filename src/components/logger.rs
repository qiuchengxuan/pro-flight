use core::fmt::{self, Display, Formatter, Write};
use core::str::from_utf8_unchecked;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::sys::jiffies;

#[derive(Copy, Clone, PartialEq)]
pub enum Level {
    Debug = 0,
    Info,
    Warning,
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

#[cfg(feature = "log-level-debug")]
const LEVEL: Level = Level::Debug;
#[cfg(feature = "log-level-info")]
const LEVEL: Level = Level::Info;
#[cfg(feature = "log-level-warning")]
const LEVEL: Level = Level::Warning;
#[cfg(feature = "log-level-error")]
const LEVEL: Level = Level::Error;

#[derive(Default)]
pub struct Logger {
    buffer: &'static mut [u8],
    index: AtomicUsize,
    writer_count: AtomicUsize,
}

impl Write for Logger {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let size = self.buffer.len();
        if size == 0 {
            return Ok(());
        }
        self.writer_count.fetch_add(1, Ordering::Relaxed);
        let index = self.index.fetch_add(1, Ordering::Relaxed);
        let mut buf = [0u8; 2];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        self.buffer[index % size] = bytes[0];
        if bytes.len() > 1 {
            self.buffer[(index + 1) % size] = bytes[1];
        }
        self.writer_count.fetch_sub(1, Ordering::Release);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer_count.fetch_add(1, Ordering::Relaxed);
        let mut bytes = s.as_bytes();
        if self.buffer.len() <= bytes.len() {
            bytes = &bytes[..self.buffer.len()];
        }
        let size = self.buffer.len();
        let index = self.index.fetch_add(bytes.len(), Ordering::Relaxed) % size;

        if size - index > bytes.len() {
            self.buffer[index..index + bytes.len()].copy_from_slice(bytes);
            self.writer_count.fetch_sub(1, Ordering::Release);
            return Ok(());
        }

        let partial_size = size - index;
        self.buffer[index..size].copy_from_slice(&bytes[..partial_size]);
        self.buffer[..bytes.len() - partial_size].copy_from_slice(&bytes[partial_size..]);
        self.writer_count.fetch_sub(1, Ordering::Release);
        Ok(())
    }
}

impl Display for Logger {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        while self.writer_count.load(Ordering::Relaxed) > 0 {}
        core::sync::atomic::fence(Ordering::Acquire);
        let index = self.index.load(Ordering::Relaxed);
        if index <= self.buffer.len() {
            return write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[..index]) });
        }
        let index = index % self.buffer.len();
        write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[index..]) })?;
        write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[..index]) })
    }
}

static mut LOGGER: Logger =
    Logger { buffer: &mut [], index: AtomicUsize::new(0), writer_count: AtomicUsize::new(0) };

#[doc(hidden)]
pub fn __write_log(args: core::fmt::Arguments, level: Level) {
    if level < LEVEL {
        return;
    }
    let logger = unsafe { &mut LOGGER };
    let jiffies = jiffies::get();
    let seconds = jiffies.as_secs() as u32;
    println!("[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), args);
    writeln!(logger, "[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), args).ok();
}

#[doc(hidden)]
pub fn __write_log_literal(message: &'static str, level: Level) {
    if level < LEVEL {
        return;
    }
    let logger = unsafe { &mut LOGGER };
    let jiffies = jiffies::get();
    let seconds = jiffies.as_secs() as u32;
    println!("[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), message);
    writeln!(logger, "[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), message).ok();
}

#[doc(hidden)]
#[macro_export]
macro_rules! __format_logger_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[macro_export]
macro_rules! log {
    ($level:path, $fmt:expr) => ({
        let _ = __format_logger_args!($fmt);
        $crate::components::logger::__write_log_literal($fmt, $level);
    });
    ($level:path, $($arg:tt)+) => {
        $crate::components::logger::__write_log(__format_logger_args!($($arg)+), $level);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Debug, $($arg)+);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Info, $($arg)+);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Warning, $($arg)+);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Error, $($arg)+);
    };
}

pub fn get() -> &'static Logger {
    unsafe { &LOGGER }
}

pub fn init(buffer: &'static mut [u8]) {
    unsafe { LOGGER = Logger { buffer, ..Default::default() } }
}
