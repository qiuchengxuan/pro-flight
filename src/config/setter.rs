use core::str::{FromStr, Split};

#[derive(Debug)]
pub enum Error {
    MalformedPath,
    ExpectValue,
    UnexpectedValue,
}

pub struct Value<'a>(pub Option<&'a str>);

impl<'a> Value<'a> {
    pub fn of(string: &'a str) -> Value<'a> {
        Value(Some(string))
    }

    pub fn parse<T: FromStr>(&self) -> Result<Option<T>, Error> {
        match self.0 {
            Some(v) => Ok(Some(FromStr::from_str(v).map_err(|_| Error::UnexpectedValue)?)),
            None => Ok(None),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let err_string = match self {
            Self::MalformedPath => "Specified path not exists or invalid",
            Self::ExpectValue => "Expected some value specified",
            Self::UnexpectedValue => "Value not valid",
        };
        write!(f, "{}", err_string)
    }
}

pub trait Setter {
    fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error>;
}
