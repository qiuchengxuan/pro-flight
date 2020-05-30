use core::fmt;

use embedded_hal::serial::{Read, Write};

use arrayvec::{Array, ArrayVec};
use ascii::{AsciiChar, ToAsciiChar};

const BACKSPACE: [u8; 3] = [AsciiChar::BackSpace as u8, ' ' as u8, AsciiChar::BackSpace as u8];

macro_rules! writes {
    ($serial:expr, $slice:expr) => {
        for &b in $slice {
            $serial.write(b).ok();
        }
    };
}

// impl<RE, WE, S: Read<u8, Error = RE> + Write<u8, Error = WE>> Console<S> {
pub fn read_line<'a, A, RE, WE, S>(serial: &mut S, vec: &'a mut ArrayVec<A>) -> Option<&'a [u8]>
where
    A: Array<Item = u8>,
    S: Read<u8, Error = RE> + Write<u8, Error = WE>,
{
    loop {
        let b = match serial.read() {
            Ok(b) => b,
            Err(_) => return None,
        };
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
            AsciiChar::ESC => continue,
            _ => {
                serial.write(b).ok();
                vec.push(b);
            }
        }
    }
}
pub fn write<WE, S: Write<u8, Error = WE>>(serial: &mut S, output: &[u8]) -> nb::Result<(), WE> {
    for &b in output.iter() {
        serial.write(b)?;
    }
    Ok(())
}

pub struct Console<'a, S>(pub &'a mut S);

impl<'a, WE, S: Write<u8, Error = WE>> fmt::Write for Console<'a, S> {
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.write(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.0.write(b as u8).ok();
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! console {
    ($serial:expr, $($arg:tt)*) => (write!(&mut Console($serial), $($arg)*).ok())
}
