use core::fmt::{Result, Write};

use btoi::btoi;

use super::yaml::{ByteStream, Entry, FromYAML, ToYAML};

#[derive(Copy, Clone, Debug)]
pub struct Battery {
    pub cells: u8,
    pub min_cell_voltage: u16,
    pub max_cell_voltage: u16,
    pub warning_cell_voltage: u16,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            cells: 0,
            min_cell_voltage: 3300,
            max_cell_voltage: 4200,
            warning_cell_voltage: 3500,
        }
    }
}

impl FromYAML for Battery {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>) {
        loop {
            match byte_stream.next(indent) {
                Some(Entry::KeyValue(key, value)) => match key {
                    b"cells" => self.cells = btoi(value).ok().unwrap_or_default(),
                    b"min-cell-voltage" => self.min_cell_voltage = btoi(value).ok().unwrap_or(3300),
                    b"max-cell-voltage" => self.max_cell_voltage = btoi(value).ok().unwrap_or(4200),
                    b"warning-cell-voltage" => {
                        self.warning_cell_voltage = btoi(value).ok().unwrap_or(3500)
                    }
                    _ => byte_stream.skip(indent),
                },
                _ => return,
            }
        }
    }
}

impl ToYAML for Battery {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        self.write_indent(indent, w)?;
        writeln!(w, "cells: {}", self.cells)?;
        self.write_indent(indent, w)?;
        writeln!(w, "min-cell-voltage: {}", self.min_cell_voltage)?;
        self.write_indent(indent, w)?;
        writeln!(w, "max-cell-voltage: {}", self.max_cell_voltage)?;
        self.write_indent(indent, w)?;
        writeln!(w, "warning-cell-voltage: {}", self.warning_cell_voltage)
    }
}

mod test {
    #[cfg(test)]
    extern crate std;

    #[test]
    fn test_write() -> core::fmt::Result {
        use std::string::String;
        use std::string::ToString;

        use super::Battery;
        use crate::config::yaml::ToYAML;

        let mut buf = String::new();
        let battery = Battery::default();
        battery.write_to(0, &mut buf)?;
        let expected = "\
        cells: 0\n\
        min-cell-voltage: 3300\n\
        max-cell-voltage: 4200\n\
        warning-cell-voltage: 3500\n\
        ";
        assert_eq!(expected, buf.to_string());
        Ok(())
    }
}
