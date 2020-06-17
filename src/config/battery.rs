use btoi::btoi;

use super::yaml::{ByteIter, Entry, FromYAML};

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
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        loop {
            match byte_iter.next(indent) {
                Entry::KeyValue(key, value) => match key {
                    b"cells" => self.cells = btoi(value).ok().unwrap_or_default(),
                    b"min-cell-voltage" => self.min_cell_voltage = btoi(value).ok().unwrap_or(3300),
                    b"max-cell-voltage" => self.max_cell_voltage = btoi(value).ok().unwrap_or(4200),
                    b"warning-cell-voltage" => {
                        self.warning_cell_voltage = btoi(value).ok().unwrap_or(3500)
                    }
                    _ => byte_iter.skip(indent),
                },
                _ => return,
            }
        }
    }
}
