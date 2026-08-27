use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parse(miniscript::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(error) => write!(f, "failed to parse miniscript: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(error) => Some(error),
        }
    }
}

impl From<miniscript::Error> for Error {
    fn from(error: miniscript::Error) -> Self {
        Error::Parse(error)
    }
}
