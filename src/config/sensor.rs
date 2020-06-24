use core::fmt::{Result, Write};

use crate::hal::sensors::Axis;

use super::yaml::{FromYAML, ToYAML, YamlParser};

#[derive(Default, Debug)]
pub struct Accelerometer {
    pub bias: Axis,
    pub gain: Axis,
}

impl FromYAML for Accelerometer {
    fn from_yaml<'a>(parser: &mut YamlParser) -> Self {
        let mut bias = Axis::default();
        let mut gain = Axis::default();
        while let Some(key) = parser.next_entry() {
            match key {
                "bias" => bias = Axis::from_yaml(parser),
                "gain" => gain = Axis::from_yaml(parser),
                _ => continue,
            }
        }
        Self { bias, gain }
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
