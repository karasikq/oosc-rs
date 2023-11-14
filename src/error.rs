#[derive(Debug)]
pub enum Error {
    Order(String),
    Generic(String),
}

impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Self::Generic(msg.into())
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::Order(err) => err.to_string(),
            Self::Generic(err) => err.to_string(),
        }
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
