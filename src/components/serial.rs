use core::fmt;

use embedded_hal::serial::Write;

pub struct SerialWritter<'a, S>(pub &'a mut S);

impl<'a, WE, S: Write<u8, Error = WE>> fmt::Write for SerialWritter<'a, S> {
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

#[macro_export]
macro_rules! write_serial {
    ($serial:expr, $message:expr) => {
        write!($crate::components::serial::SerialWritter($serial), "{}", $message)
    };
}
