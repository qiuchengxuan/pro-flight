use heapless::Vec;

use super::command;
use crate::config::pathset::{Error, Path, PathClear, PathSet, Value};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Toggle {
    pub channel: u8,
    pub choices: Vec<command::Id, 3>,
}

impl PathSet for Toggle {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        match path.str()? {
            "channel" => {
                self.channel = value.parse()?;
            }
            "choices" => {
                let index: usize = path.parse()?;
                let value: command::Id = value.parse()?;
                match index {
                    _ if index < self.choices.len() => self.choices[index] = value,
                    _ if index == self.choices.len() => {
                        self.choices.push(value).map_err(|_| Error::UnknownPath)?;
                    }
                    _ => return Err(Error::UnknownPath),
                }
            }
            _ => return Err(Error::UnknownPath),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Toggles(pub Vec<Toggle, 4>);

impl Toggles {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PathSet for Toggles {
    fn set(&mut self, mut path: Path, value: Value) -> Result<(), Error> {
        let index: usize = path.parse()?;
        match index {
            _ if index > self.0.len() => return Err(Error::UnknownPath),
            _ if index == self.0.len() => {
                self.0.push(Toggle::default()).map_err(|_| Error::UnknownPath)?;
                self.0[index].set(path, value)
            }
            _ => self.0[index].set(path, value),
        }
    }
}

impl PathClear for Toggles {
    fn clear(&mut self, mut path: Path) -> Result<(), Error> {
        let index: usize = path.parse()?;
        if index == self.0.len() && self.0.len() > 0 {
            self.0.pop();
            return Ok(());
        }
        Err(Error::UnknownPath)
    }
}
