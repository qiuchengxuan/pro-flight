use super::pathset::{Error, Path, PathClear, PathSet, Value};

mod axes;
pub mod command;
mod toggles;

pub use axes::Axes;
pub use toggles::Toggles;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inputs {
    pub axes: Axes,
    #[serde(skip_serializing_if = "Toggles::is_empty")]
    pub toggles: Toggles,
}

impl PathSet for Inputs {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "axes" => self.axes.set(path, value),
            "toggles" => self.toggles.set(path, value),
            _ => Err(Error::UnknownPath),
        }
    }
}

impl PathClear for Inputs {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        match path.str()? {
            "axes" => self.axes.clear(path),
            "toggles" => self.toggles.clear(path),
            _ => Err(Error::UnknownPath),
        }
    }
}
