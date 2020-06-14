use super::yaml::{ByteIter, Entry, FromYAML};
use crate::hal::sensors::Axes;

#[derive(Default, Debug)]
pub struct Accelerometer {
    pub bias: Axes,
    pub gain: Axes,
}

impl FromYAML for Accelerometer {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>) {
        for _ in 0..2 {
            match byte_iter.next(indent) {
                Entry::Key(key) => match key {
                    b"bias" => self.bias.from_yaml(indent + 2, byte_iter),
                    b"gain" => self.gain.from_yaml(indent + 2, byte_iter),
                    _ => return,
                },
                _ => return,
            }
        }
    }
}
