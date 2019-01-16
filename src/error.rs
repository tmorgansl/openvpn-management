use std::error::Error;
use std::fmt;
use std::io;
use std::net;
use std::num;

pub type OpenvpnResult<T> = Result<T, OpenvpnError>;

#[derive(Debug)]
pub enum OpenvpnError {
    Io(io::Error),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
    ParseAddr(net::AddrParseError),
    MalformedResponse(String),
}

impl fmt::Display for OpenvpnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpenvpnError::Io(ref err) => err.fmt(f),
            OpenvpnError::ParseInt(ref err) => err.fmt(f),
            OpenvpnError::ParseFloat(ref err) => err.fmt(f),
            OpenvpnError::ParseAddr(ref err) => err.fmt(f),
            OpenvpnError::MalformedResponse(ref response) => write!(
                f,
                "could not parse '{}' response from openvpn server",
                response
            ),
        }
    }
}

impl Error for OpenvpnError {
    fn description(&self) -> &str {
        match *self {
            OpenvpnError::Io(ref err) => err.description(),
            OpenvpnError::ParseInt(ref err) => err.description(),
            OpenvpnError::ParseFloat(ref err) => err.description(),
            OpenvpnError::ParseAddr(ref err) => err.description(),
            OpenvpnError::MalformedResponse(ref _response) => "malformed response",
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

impl From<net::AddrParseError> for OpenvpnError {
    fn from(err: net::AddrParseError) -> OpenvpnError {
        OpenvpnError::ParseAddr(err)
    }
}
