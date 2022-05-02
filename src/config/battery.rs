use crate::types::measurement::voltage::Voltage;

use super::pathset::{Error, Path, PathSet, Value};

const DEFAULT_MIN_CELL_VOLTAGE: Voltage = voltage!(3.3);
const DEFAULT_MAX_CELL_VOLTAGE: Voltage = voltage!(4.2);
const DEFAULT_WARNING_CELL_VOLTAGE: Voltage = voltage!(3.5);

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Battery {
    pub cells: u8,
    pub min_cell_voltage: Voltage,
    pub max_cell_voltage: Voltage,
    pub warning_cell_voltage: Voltage,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            cells: 3,
            min_cell_voltage: DEFAULT_MIN_CELL_VOLTAGE,
            max_cell_voltage: DEFAULT_MAX_CELL_VOLTAGE,
            warning_cell_voltage: DEFAULT_WARNING_CELL_VOLTAGE,
        }
    }
}

impl PathSet for Battery {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "cells" => self.cells = value.parse_or(3)?,
            "min-cell-voltage" => {
                self.min_cell_voltage = value.parse_or(DEFAULT_MIN_CELL_VOLTAGE)?
            }
            "max-cell-voltage" => {
                self.max_cell_voltage = value.parse_or(DEFAULT_MAX_CELL_VOLTAGE)?
            }
            "warning-cell-voltage" => {
                self.warning_cell_voltage = value.parse_or(DEFAULT_WARNING_CELL_VOLTAGE)?
            }
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}
