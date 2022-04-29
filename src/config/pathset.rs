use core::str::{FromStr, Split};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    UnknownPath,
    ExpectValue,
    InvalidValue,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value<'a>(pub Option<&'a str>);

impl<'a> Value<'a> {
    pub fn of(string: &'a str) -> Value<'a> {
        Value(Some(string))
    }

    pub fn str(&self) -> Result<&'a str, Error> {
        self.0.ok_or(Error::InvalidValue)
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, Error> {
        match self.0 {
            Some(s) => FromStr::from_str(s).map_err(|_| Error::InvalidValue),
            None => Err(Error::InvalidValue),
        }
    }

    pub fn parse_or<T: FromStr>(&self, or: T) -> Result<T, Error> {
        match self.0 {
            Some(s) => FromStr::from_str(s).map_err(|_| Error::InvalidValue),
            None => Ok(or),
        }
    }

    pub fn parse_or_default<T: FromStr + Default>(&self) -> Result<T, Error> {
        match self.0 {
            Some(s) => FromStr::from_str(s).map_err(|_| Error::InvalidValue),
            None => Ok(T::default()),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let err_string = match self {
            Self::UnknownPath => "Specified path not exists or invalid",
            Self::ExpectValue => "Expect value",
            Self::InvalidValue => "Value not valid",
        };
        write!(f, "{}", err_string)
    }
}

#[derive(Clone, Debug)]
pub struct Path<'a>(Split<'a, char>);

impl<'a> Path<'a> {
    pub fn new(split: Split<'a, char>) -> Self {
        Self(split)
    }

    pub fn str(&mut self) -> Result<&str, Error> {
        self.0.next().ok_or(Error::UnknownPath)
    }

    pub fn parse<T: FromStr>(&mut self) -> Result<T, Error> {
        match self.0.next() {
            Some(s) => T::from_str(s).map_err(|_| Error::UnknownPath),
            None => Err(Error::UnknownPath),
        }
    }

    pub fn unwrap(self) -> Split<'a, char> {
        self.0
    }
}

pub trait PathSet {
    fn set(&mut self, path: Path, value: Value) -> Result<(), Error>;
}

pub trait PathClear {
    fn clear(&mut self, path: Path) -> Result<(), Error>;
}
