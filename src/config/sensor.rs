use core::fmt::{Result, Write};

use crate::hal::sensors::Axis;

use super::yaml::{ByteStream, Entry, FromYAML, ToYAML};

#[derive(Default, Debug)]
pub struct Accelerometer {
    pub bias: Axis,
    pub gain: Axis,
}

impl FromYAML for Accelerometer {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::Key(key)) => match key {
                    b"bias" => self.bias.from_yaml(indent + 1, byte_stream),
                    b"gain" => self.gain.from_yaml(indent + 1, byte_stream),
                    _ => byte_stream.skip(indent),
                },
                _ => return,
            }
        }
    }
}

impl ToYAML for Accelerometer {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "bias:")?;
        self.bias.write_to(indent + 1, w)?;

        self.write_indent(indent, w)?;
        writeln!(w, "gain:")?;
        self.gain.write_to(indent + 1, w)
    }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    fn test_write() -> core::fmt::Result {
        use std::string::{String, ToString};

        use super::Accelerometer;
        use crate::config::yaml::ToYAML;

        let mut buf = String::new();
        let accelerometer = Accelerometer::default();
        accelerometer.write_to(0, &mut buf)?;
        let expected = "bias:\n  x: 0\n  y: 0\n  z: 0\ngain:\n  x: 0\n  y: 0\n  z: 0";
        assert_eq!(expected.trim(), buf.to_string().trim());
        Ok(())
    }
}
