use core::fmt::{Display, Result, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

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

static mut LOG_BUFFER: &'static mut [u8] = &mut [0u8; 0];
static mut WRITE_INDEX: AtomicUsize = AtomicUsize::new(0);
static mut LEVEL: Level = Level::Debug;

pub struct Logger;

impl Logger {
    fn allocate(&self, len: usize) -> usize {
        let mut index = unsafe { WRITE_INDEX.load(Ordering::Acquire) };
        loop {
            let new_index = index + len;
            let current =
                unsafe { WRITE_INDEX.compare_and_swap(index, new_index, Ordering::SeqCst) };
            if current == index {
                return index as usize;
            }
            index = current;
        }
    }
}

impl Write for Logger {
    fn write_char(&mut self, c: char) -> Result {
        let buffer = unsafe { &mut LOG_BUFFER };
        if buffer.len() == 0 {
            return Ok(());
        }
        let index = self.allocate(1);
        buffer[index % buffer.len()] = c as u8;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result {
        let buffer = unsafe { &mut LOG_BUFFER };
        let bytes = s.as_bytes();
        if buffer.len() <= bytes.len() {
            return Ok(());
        }
        let index = self.allocate(bytes.len()) % buffer.len();
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
pub fn __write_log(args: core::fmt::Arguments, level: Level, file: &'static str, line: u32) {
    let mut logger = Logger {};
    let filename = file.rsplitn(2, "/").next().unwrap_or("?.rs");
    write!(logger, "{} {}:{}\t", level, filename, line).ok();
    writeln!(logger, "{}", args).ok();
}

#[doc(hidden)]
pub fn __write_log_literal(message: &'static str, level: Level, file: &'static str, line: u32) {
    let mut logger = Logger {};
    let filename = file.rsplitn(2, "/").next().unwrap_or("?.rs");
    write!(logger, "{} {}:{}\t", level, filename, line).ok();
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
        $crate::components::logger::__write_log_literal($message, $level, file!(), line!());
    });
    ($level:expr, $($arg:tt)+) => {
        $crate::components::logger::__write_log(__format_args!($($arg)+), $level, file!(), line!());
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

pub fn init(buffer: &'static mut [u8], level: Level) {
    unsafe {
        LOG_BUFFER = buffer;
        LEVEL = level
    }
}

pub struct LogReader((usize, usize));

impl Iterator for LogReader {
    type Item = &'static [u8];

    fn next(&mut self) -> Option<&'static [u8]> {
        let (index, count) = self.0;
        let log_buffer = unsafe { &LOG_BUFFER };
        if index <= log_buffer.len() {
            if count == 0 {
                self.0 = (index, 1);
                return Some(unsafe { &LOG_BUFFER[..index] });
            } else {
                return None;
            }
        }
        if count == 0 {
            self.0 = (index, 1);
            let bytes = unsafe { &LOG_BUFFER[index % log_buffer.len()..] };
            return bytes.splitn(1, |&b| b == '\n' as u8).next();
        } else if count == 1 {
            self.0 = (index, 2);
            return Some(unsafe { &LOG_BUFFER[..index % log_buffer.len()] });
        }
        None
    }
}

pub fn reader() -> LogReader {
    unsafe {
        let index = WRITE_INDEX.load(Ordering::Acquire);
        LogReader((index, 0))
    }
}
