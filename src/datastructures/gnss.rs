#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FixType {
    NoFix,
    TwoDemension,
    ThreeDemension,
}

impl Into<&str> for FixType {
    fn into(self) -> &'static str {
        match self {
            Self::NoFix => "no-fix",
            Self::TwoDemension => "2D",
            Self::ThreeDemension => "3D",
        }
    }
}

impl sval::value::Value for FixType {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.str((*self).into())
    }
}

impl Default for FixType {
    fn default() -> Self {
        Self::NoFix
    }
}
