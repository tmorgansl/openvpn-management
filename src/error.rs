use std::error::Error;
use std::fmt;
use std::io;
use std::num;

pub type OpenvpnResult<T> = Result<T, OpenvpnError>;

#[derive(Debug)]
pub enum OpenvpnError {
    Io(io::Error),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
    MalformedResponse(String),
    MissingURLInput(String),
}

impl fmt::Display for OpenvpnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpenvpnError::Io(ref err) => err.fmt(f),
            OpenvpnError::ParseInt(ref err) => err.fmt(f),
            OpenvpnError::ParseFloat(ref err) => err.fmt(f),
            OpenvpnError::MalformedResponse(ref response) => write!(
                f,
                "could not parse '{}' response from openvpn server",
                response
            ),
            OpenvpnError::MissingURLInput(ref url) => {
                write!(f, "could not parse '{}' as a URL", url)
            }
        }
    }
}

impl Error for OpenvpnError {
    fn description(&self) -> &str {
        match *self {
            OpenvpnError::Io(ref err) => err.description(),
            OpenvpnError::ParseInt(ref err) => err.description(),
            OpenvpnError::ParseFloat(ref err) => err.description(),
            OpenvpnError::MalformedResponse(ref _response) => "malformed response",
            OpenvpnError::MissingURLInput(ref _url) => "missing url",
        }
    }
}

impl From<io::Error> for OpenvpnError {
    fn from(err: io::Error) -> OpenvpnError {
        OpenvpnError::Io(err)
    }
}

impl From<num::ParseIntError> for OpenvpnError {
    fn from(err: num::ParseIntError) -> OpenvpnError {
        OpenvpnError::ParseInt(err)
    }
}

impl From<num::ParseFloatError> for OpenvpnError {
    fn from(err: num::ParseFloatError) -> OpenvpnError {
        OpenvpnError::ParseFloat(err)
    }
}
