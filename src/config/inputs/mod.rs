use super::pathset::{Error, Path, PathClear, PathSet, Value};

mod axes;
mod command;

pub use axes::Axes;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inputs {
    pub axes: Axes,
}

impl PathSet for Inputs {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "axes" => self.axes.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}

impl PathClear for Inputs {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        match path.str()? {
            "axes" => self.axes.clear(path),
            _ => Err(Error::UnknownPath),
        }
    }
}
