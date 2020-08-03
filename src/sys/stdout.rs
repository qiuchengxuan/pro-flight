use alloc::rc::Rc;
use core::fmt::Write;

use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};

static mut STDOUT: Option<OverwritingData<u8>> = None;

#[doc(hidden)]
pub fn __write_stdout(args: core::fmt::Arguments) {
    if let Some(stdout) = unsafe { STDOUT.as_mut() } {
        write!(stdout, "{}", args).ok();
    }
}

#[doc(hidden)]
pub fn __write_stdout_literal(message: &'static str) {
    if let Some(stdout) = unsafe { STDOUT.as_mut() } {
        write!(stdout, "{}", message).ok();
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __format_stdout_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[macro_export]
macro_rules! print {
    ($message:expr) => ({
        let _ = __format_stdout_args!($message);
        $crate::sys::stdout::__write_stdout_literal($message);
    });
    ($($arg:tt)+) => {
        $crate::sys::stdout::__write_stdout(__format_stdout_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! println {
    ($message:expr) => ({
        let _ = __format_stdout_args!($message);
        $crate::sys::stdout::__write_stdout_literal($message);
        $crate::sys::stdout::__write_stdout_literal("\r\n");
    });
    ($($arg:tt)+) => ({
        $crate::sys::stdout::__write_stdout(__format_stdout_args!($($arg)+));
        $crate::sys::stdout::__write_stdout_literal("\r\n");
    });
}

pub fn reader() -> OverwritingDataSource<u8> {
    let stdout = unsafe { &Rc::from_raw(STDOUT.as_ref().unwrap()) };
    core::mem::forget(stdout);
    OverwritingDataSource::new(stdout)
}

pub fn init(size: usize) {
    unsafe { STDOUT = Some(OverwritingData::sized(size)) }
}
