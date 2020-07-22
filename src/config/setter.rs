use core::str::Split;

#[derive(Debug)]
pub enum SetError {
    MalformedPath,
    ExpectValue,
    UnexpectedValue,
}

impl core::fmt::Display for SetError {
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
    fn set(&mut self, path: &mut Split<char>, value: Option<&str>) -> Result<(), SetError>;
}
