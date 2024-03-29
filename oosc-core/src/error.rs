// Use this only in not nested features??
// And anyhow in other cases
#[derive(Debug)]
pub enum Error {
    Order(String),
    Option(&'static str),
    OutOfRange(usize, usize),
    Specify(&'static str),
    Generic(String),
}

impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Self::Generic(msg.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<&'static str> for Error {
    fn from(err: &'static str) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Generic(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Order(err) => write!(f, "{}", err),
            Self::Option(err) => write!(f, "Cannot take {} option", err),
            Self::OutOfRange(index, range) => write!(
                f,
                "Buffer index out of range. Found {}, expected <= {}",
                index, range
            ),
            Self::Generic(err) => write!(f, "{}", err),
            Self::Specify(err) => write!(f, "Need to specify {} first", err),
        }
    }
}

impl std::error::Error for Error {}
