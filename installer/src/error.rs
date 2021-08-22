use std::{fmt::Display, path::PathBuf};

#[derive(Debug)]
pub(crate) enum Error {
    IoError(std::io::Error),
    CommandFailed {
        command: PathBuf,
        stdout: String,
        stderr: String,
    },
    NwgError(nwg::NwgError),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<nwg::NwgError> for Error {
    fn from(error: nwg::NwgError) -> Self {
        Self::NwgError(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(error) => Display::fmt(error, f),
            Error::CommandFailed { command, ..} => {
                write!(f, "Nepavyko įvykdyti komandos: {:?}", command)
            },
            Error::NwgError(error) => Display::fmt(error, f),
        }
    }
}

pub(crate) type IResult<T=()> = Result<T, Error>;