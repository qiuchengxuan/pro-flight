use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write as _;

use embedded_hal::serial::{Read, Write};

use ascii::{AsciiChar, ToAsciiChar};

const BACKSPACE: [u8; 3] = [AsciiChar::BackSpace as u8, ' ' as u8, AsciiChar::BackSpace as u8];

macro_rules! writes {
    ($serial:expr, $slice:expr) => {
        for &b in $slice {
            $serial.write(b).ok();
        }
    };
}

pub fn read_line<'a, RE, WE, S>(serial: &mut S, vec: &'a mut Vec<u8>) -> Option<&'a [u8]>
where
    S: Read<u8, Error = RE> + Write<u8, Error = WE>,
{
    let mut skip = false;
    loop {
        let b = match serial.read() {
            Ok(b) => b,
            Err(_) => return None,
        };
        if skip {
            skip = false;
            continue;
        }
        match unsafe { b.to_ascii_char_unchecked() } {
            AsciiChar::BackSpace => {
                if let Some(_) = vec.pop() {
                    writes!(serial, &BACKSPACE);
                }
            }
            AsciiChar::DEL => {
                if let Some(_) = vec.pop() {
                    writes!(serial, &BACKSPACE);
                }
            }
            AsciiChar::CarriageReturn => {
                writes!(serial, b"\r\n");
                return Some(vec.as_slice());
            }
            AsciiChar::ETB => {
                // ^W or CTRL+W
                while vec.len() > 0 {
                    if vec.pop().unwrap() == ' ' as u8 {
                        break;
                    }
                    writes!(serial, &BACKSPACE);
                }
            }
            AsciiChar::ESC => {
                skip = true;
                continue;
            }
            _ => {
                vec.push(b);
                serial.write(b).ok();
            }
        }
    }
}

pub fn write<WE, S: Write<u8, Error = WE>>(serial: &mut S, output: &[u8]) -> nb::Result<(), WE> {
    for &b in output.iter() {
        if b == '\n' as u8 {
            serial.write('\r' as u8)?;
        }
        serial.write(b)?;
    }
    Ok(())
}

pub struct Console<'a, S>(pub &'a mut S);

impl<'a, WE, S: Write<u8, Error = WE>> fmt::Write for Console<'a, S> {
    fn write_char(&mut self, c: char) -> fmt::Result {
        if c == '\n' {
            self.0.write('\r' as u8).ok();
        }
        self.0.write(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                self.0.write('\r' as u8).ok();
            }
            self.0.write(b as u8).ok();
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn __write_console<E, S: Write<u8, Error = E>>(serial: &mut S, args: core::fmt::Arguments) {
    write!(&mut Console(serial), "{}", args).ok();
}

#[doc(hidden)]
pub fn __write_console_literal<E, S: Write<u8, Error = E>>(serial: &mut S, message: &'static str) {
    write!(&mut Console(serial), "{}", message).ok();
}

#[macro_export]
macro_rules! console {
    ($serial:expr, $message:expr) => ({
        let _ = __format_args!($message); // XXX: defined in logger.rs
        $crate::components::console::__write_console_literal($serial, $message);
    });
    ($serial:expr, $($arg:tt)+) => {
        $crate::components::console::__write_console($serial, __format_args!($($arg)+));
    };
}
