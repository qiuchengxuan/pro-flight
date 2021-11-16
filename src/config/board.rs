use core::str::Split;

use crate::{
    config::setter::{Error, Setter, Value},
    datastructures::measurement::Rotation,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize)]
pub struct Board {
    pub rotation: Rotation,
}

impl Setter for Board {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
        match path.next().ok_or(Error::MalformedPath)? {
            "rotation" => self.rotation = value.parse()?.unwrap_or(Rotation::NoRotation),
            _ => return Err(Error::MalformedPath),
        }
        Ok(())
    }
}
