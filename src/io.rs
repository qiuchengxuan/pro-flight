use core::cell::Cell;
use core::fmt::{self, Write as _};
use core::sync::atomic::{AtomicBool, Ordering};

pub trait Read {
    type Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait Write {
    type Error;
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub enum Error {
    Locked,
    Unknown,
}

extern "Rust" {
    fn stdout_write_bytes(bytes: &[u8]);
    fn stdout_flush();
    fn stdin_read_bytes(buffer: &mut [u8]) -> Result<usize, Error>;
}

static STDIN_LOCK: AtomicBool = AtomicBool::new(false);

pub struct Stdin(Cell<bool>);

pub fn stdin() -> Stdin {
    Stdin(Cell::new(false))
}

impl Stdin {
    pub fn lock(&self) -> bool {
        if STDIN_LOCK
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            self.0.set(true);
            return true;
        }
        false
    }

    pub fn unlock(&self) {
        if self.0.take() {
            STDIN_LOCK.store(false, Ordering::Relaxed);
        }
    }
}

impl Drop for Stdin {
    fn drop(&mut self) {
        self.unlock()
    }
}

impl Read for Stdin {
    type Error = Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.0.get() {
            return unsafe { stdin_read_bytes(buf) };
        }
        if !self.lock() {
            return Err(Error::Locked);
        }
        let result = unsafe { stdin_read_bytes(buf) };
        self.unlock();
        result
    }
}

pub struct Stdout;

pub fn stdout() -> Stdout {
    Stdout
}

impl Write for Stdout {
    type Error = Error;
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        unsafe { stdout_write_bytes(bytes) };
        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        unsafe { stdout_flush() };
        Ok(())
    }
}

impl fmt::Write for Stdout {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let mut buffer = [0u8; 2];
        match self.write(c.encode_utf8(&mut buffer).as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[doc(hidden)]
pub fn __write_stdout_literal(fmt: &str) {
    write!(stdout(), "{}", fmt).ok();
}

#[doc(hidden)]
pub fn __write_stdout(args: fmt::Arguments) {
    write!(stdout(), "{}", args).ok();
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! print {
    ($fmt:expr) => {
        $crate::io::__write_stdout_literal($fmt)
    };
    ($($args:tt)+) => {
        $crate::io::__write_stdout(format_args!($($args)+))
    };
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => {
        print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($args:tt)+) => {
        print!(concat!($fmt, "\n"), $($args)+)
    };
}
