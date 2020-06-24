use core::fmt::{Result, Write};

use btoi::btoi;

use super::yaml::{FromYAML, ToYAML, YamlParser};

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
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut cells: u8 = 0;
        let mut min_cell_voltage: u16 = 3300;
        let mut max_cell_voltage: u16 = 4200;
        let mut warning_cell_voltage: u16 = 3500;
        while let Some((key, value)) = parser.next_key_value() {
            let value = btoi::<u16>(value.as_bytes()).ok();
            match key {
                "cells" => cells = value.unwrap_or(0) as u8,
                "min-cell-voltage" => min_cell_voltage = value.unwrap_or(3300),
                "max-cell-voltage" => max_cell_voltage = value.unwrap_or(4200),
                "warning-cell-voltage" => warning_cell_voltage = value.unwrap_or(3500),
                _ => continue,
            }
        }
        Self { cells, min_cell_voltage, max_cell_voltage, warning_cell_voltage }
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
