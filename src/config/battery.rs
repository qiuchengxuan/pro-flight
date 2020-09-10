use core::fmt::{Result, Write};

use crate::datastructures::decimal::IntegerDecimal;

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Copy, Clone, Debug)]
pub struct Battery {
    pub cells: u8,
    pub min_cell_voltage: IntegerDecimal<u8, u8>,
    pub max_cell_voltage: IntegerDecimal<u8, u8>,
    pub warning_cell_voltage: IntegerDecimal<u8, u8>,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            cells: 0,
            min_cell_voltage: IntegerDecimal::from("3.3"),
            max_cell_voltage: IntegerDecimal::from("4.2"),
            warning_cell_voltage: IntegerDecimal::from("3.5"),
        }
    }
}

impl FromYAML for Battery {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self {
        let mut cells: u8 = 0;
        let mut min_cell_voltage = IntegerDecimal::from("3.3");
        let mut max_cell_voltage = IntegerDecimal::from("4.2");
        let mut warning_cell_voltage = IntegerDecimal::from("3.5");
        while let Some((key, value)) = parser.next_key_value() {
            match key {
                "cells" => cells = value.parse().unwrap_or(0),
                "min-cell-voltage" => min_cell_voltage = IntegerDecimal::from(value),
                "max-cell-voltage" => max_cell_voltage = IntegerDecimal::from(value),
                "warning-cell-voltage" => warning_cell_voltage = IntegerDecimal::from(value),
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
