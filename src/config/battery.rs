use core::fmt::Write;
use core::str::Split;

use crate::datastructures::fixed_point::FixedPoint;

use super::setter::{Error, Setter, Value};
use super::yaml::ToYAML;

const DEFAULT_MIN_CELL_VOLTAGE: FixedPoint<i32, 3> = FixedPoint(3_300);
const DEFAULT_MAX_CELL_VOLTAGE: FixedPoint<i32, 3> = FixedPoint(4_200);
const DEFAULT_WARNING_CELL_VOLTAGE: FixedPoint<i32, 3> = FixedPoint(3_500);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Battery {
    pub cells: u8,
    pub min_cell_voltage: FixedPoint<i32, 3>,
    pub max_cell_voltage: FixedPoint<i32, 3>,
    pub warning_cell_voltage: FixedPoint<i32, 3>,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            cells: 0,
            min_cell_voltage: DEFAULT_MIN_CELL_VOLTAGE,
            max_cell_voltage: DEFAULT_MAX_CELL_VOLTAGE,
            warning_cell_voltage: DEFAULT_WARNING_CELL_VOLTAGE,
        }
    }
}

impl Setter for Battery {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "cells" => self.cells = value.parse()?.unwrap_or(0),
            "min-cell-voltage" => {
                self.min_cell_voltage = value.parse()?.unwrap_or(DEFAULT_MIN_CELL_VOLTAGE)
            }
            "max-cell-voltage" => {
                self.max_cell_voltage = value.parse()?.unwrap_or(DEFAULT_MAX_CELL_VOLTAGE)
            }
            "warning-cell-voltage" => {
                self.warning_cell_voltage = value.parse()?.unwrap_or(DEFAULT_WARNING_CELL_VOLTAGE)
            }
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}

impl ToYAML for Battery {
    fn write_to(&self, indent: usize, w: &mut impl Write) -> core::fmt::Result {
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
