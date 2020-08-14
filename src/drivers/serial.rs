use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write as _;

use embedded_hal::serial;

use ascii::{AsciiChar, ToAsciiChar};

use crate::hal::io;

impl<'a, S: serial::Write<u8>> io::Write for S {
    type Error = S::Error;

    fn write(&mut self, bytes: &[u8]) -> Result<usize, S::Error> {
        for &b in bytes.iter() {
            nb::block!(self.write(b))?;
        }
        Ok(bytes.len())
    }
}

pub struct Serial<RE, WE, T: serial::Write<u8, Error = WE> + serial::Read<u8, Error = RE>>(pub T);

impl<'a, RE, WE, T> fmt::Write for Serial<RE, WE, T>
where
    T: serial::Write<u8, Error = WE> + serial::Read<u8, Error = RE>,
{
    fn write_char(&mut self, c: char) -> fmt::Result {
        let mut buf = [0u8; 2];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        for &b in bytes.iter() {
            if b == '\n' as u8 {
                nb::block!(self.0.write('\r' as u8)).ok();
            }
            nb::block!(self.0.write(b)).ok();
        }
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes().iter() {
            if b == '\n' as u8 {
                nb::block!(self.0.write('\r' as u8)).ok();
            }
            nb::block!(self.0.write(b)).ok();
        }
        Ok(())
    }
}

pub trait Readline {
    fn readline<'a>(&mut self, vec: &'a mut Vec<u8>) -> Option<&'a [u8]>;
}

const BACKSPACE: &str = "\x08 \x08";

impl<RE, WE, T> Readline for Serial<RE, WE, T>
where
    T: serial::Write<u8, Error = WE> + serial::Read<u8, Error = RE>,
{
    fn readline<'a>(&mut self, vec: &'a mut Vec<u8>) -> Option<&'a [u8]> {
        let mut skip = false;
        loop {
            let b = match self.0.read() {
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
                        write!(self, "{}", &BACKSPACE).ok();
                    }
                }
                AsciiChar::DEL => {
                    if let Some(_) = vec.pop() {
                        write!(self, "{}", &BACKSPACE).ok();
                    }
                }
                AsciiChar::CarriageReturn => {
                    writeln!(self, "").ok();
                    return Some(vec.as_slice());
                }
                AsciiChar::ETB => {
                    // ^W or CTRL+W
                    while vec.len() > 0 {
                        if vec.pop().unwrap() == ' ' as u8 {
                            break;
                        }
                        write!(self, "{}", &BACKSPACE).ok();
                    }
                }
                AsciiChar::ESC => {
                    skip = true;
                    continue;
                }
                _ => {
                    vec.push(b);
                    self.0.write(b).ok();
                }
            }
        }
    }
}
