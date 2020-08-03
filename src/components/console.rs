use alloc::vec::Vec;

use embedded_hal::serial::{Read, Write};

use ascii::{AsciiChar, ToAsciiChar};

const BACKSPACE: &str = "\x08 \x08";

pub fn readline<'a, RE, WE, S>(serial: &mut S, vec: &'a mut Vec<u8>) -> Option<&'a [u8]>
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
                    print!("{}", BACKSPACE);
                }
            }
            AsciiChar::DEL => {
                if let Some(_) = vec.pop() {
                    print!("{}", BACKSPACE);
                }
            }
            AsciiChar::CarriageReturn => {
                println!("");
                return Some(vec.as_slice());
            }
            AsciiChar::ETB => {
                // ^W or CTRL+W
                while vec.len() > 0 {
                    if vec.pop().unwrap() == ' ' as u8 {
                        break;
                    }
                    print!("{}", BACKSPACE);
                }
            }
            AsciiChar::ESC => {
                skip = true;
                continue;
            }
            _ => {
                vec.push(b);
                print!("{}", b);
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
