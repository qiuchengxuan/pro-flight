use alloc::vec::Vec;

use ascii::{AsciiChar, ToAsciiChar};

pub struct Terminal {
    buffer: Vec<u8>,
    eol: bool,
}

const BACKSPACE: &str = "\x08 \x08";

impl Terminal {
    pub fn new() -> Self {
        Self { buffer: Vec::with_capacity(80), eol: false }
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Option<&str> {
        if self.eol {
            self.eol = false;
            self.buffer.truncate(0);
        }
        for &b in bytes.iter() {
            match unsafe { b.to_ascii_char_unchecked() } {
                AsciiChar::BackSpace | AsciiChar::DEL => {
                    if let Some(_) = self.buffer.pop() {
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::CarriageReturn => {
                    self.eol = true;
                    println!("");
                    return Some(unsafe { core::str::from_utf8_unchecked(self.buffer.as_slice()) });
                }
                AsciiChar::ETB => {
                    // ^W or CTRL+W
                    loop {
                        match self.buffer.pop() {
                            Some(ch) => {
                                print!("{}", &BACKSPACE);
                                if ch == ' ' as u8 {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }
                ch if ch.is_ascii_printable() => {
                    self.buffer.push(b);
                    print!("{}", ch);
                }
                _ => (),
            }
        }
        None
    }
}
