//! Error type

use std::error::Error as StdError;
use std::fmt::Display;

use std::io::Error as IoError;
use std::ffi::NulError;
use std::sync::PoisonError;

/// srcds `Result` alias type.
pub type Result<T> = ::std::result::Result<T, Error>;

/// srcds error type.
#[derive(Debug)]
pub enum Error {
    /// An `std::io` module error.
    Io(IoError),
    /// A `std::ffi::NulError` wrapper.
    Nul(NulError),
    /// A thread holding a locked mutex panicked and poisoned the lock.
    Poison,
    /// Response received is invalid.
    InvalidResponse,
    /// A miscellaneous error with a description
    Other(&'static str),
}

macro_rules! from_error {
    ($orig:ty, $mapping:ident) => {
        impl From<$orig> for Error {
            fn from(err: $orig) -> Error {
                Error::$mapping(err)
            }
        }
    };
}

from_error!(IoError, Io);
from_error!(NulError, Nul);

impl<Guard> From<PoisonError<Guard>> for Error {
    fn from(_: PoisonError<Guard>) -> Error {
        Error::Poison
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::Io(ref inner) => inner.fmt(f),
            Error::Nul(ref inner) => inner.fmt(f),
            _ => f.write_str(self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref inner) => inner.description(),
            Error::Nul(ref inner) => inner.description(),
            Error::Poison => "Mutex has been poisoned",
            Error::InvalidResponse => "Invalid response",
            Error::Other(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref inner) => Some(inner),
            Error::Nul(ref inner) => Some(inner),
            _ => None,
        }
    }
}