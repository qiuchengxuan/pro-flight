use core::cell::RefCell;

use arrayvec::{Array, ArrayVec};
use ascii::{AsciiChar, ToAsciiChar};

pub trait Writable {
    fn write(&mut self, output: &[u8]);
}

pub trait ReadSome {
    fn read_some<'a>(&mut self, input: &'a mut [u8]) -> &'a [u8];
}

pub trait Available {
    fn available(&mut self) -> bool;
}

pub struct Console<'a, U> {
    serial: RefCell<&'a mut U>,
}

const BACKSPACE: [u8; 3] = [
    AsciiChar::BackSpace as u8,
    ' ' as u8,
    AsciiChar::BackSpace as u8,
];

impl<'a, U: Writable + ReadSome + Available> Console<'a, U> {
    pub fn new(serial: &'a mut U) -> Self {
        Self {
            serial: RefCell::new(serial),
        }
    }

    pub fn try_read_line<'b, A>(&self, vec: &'b mut ArrayVec<A>) -> Option<&'b [u8]>
    where
        A: Array<Item = u8>,
    {
        let mut serial = self.serial.borrow_mut();
        if !serial.available() {
            return None;
        }

        let mut buf = [0u8; 10];
        for token in serial.read_some(&mut buf).iter() {
            match unsafe { token.to_ascii_char_unchecked() } {
                AsciiChar::BackSpace => {
                    if let Some(_) = vec.pop() {
                        serial.write(&BACKSPACE);
                    }
                }
                AsciiChar::DEL => {
                    if let Some(_) = vec.pop() {
                        serial.write(&BACKSPACE);
                    }
                }
                AsciiChar::CarriageReturn => {
                    serial.write(b"\r\n");
                    return Some(vec.as_slice());
                }
                AsciiChar::ETB => {
                    // ^W or CTRL+W
                    while vec.len() > 0 {
                        if vec.pop().unwrap() == ' ' as u8 {
                            break;
                        }
                        serial.write(&BACKSPACE);
                    }
                }
                _ => {
                    serial.write(&[*token]);
                    vec.push(*token);
                }
            }
        }
        None
    }

    pub fn write<'b>(&self, output: &'b [u8]) {
        self.serial.borrow_mut().write(output)
    }
}
