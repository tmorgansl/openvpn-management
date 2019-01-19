//! openvpn-management is a wrapper to the [openvpn management interface](https://openvpn.net/community-resources/management-interface/).
//! # Install:
//! The crate is called `openvpn-management` and you can depend on it via cargo:
//! ```ini
//! [dependencies]
//! openvpn-management = "*"
//! ```
//! # Features:
//! - Getting all connected client information
//! # Basic usage:
//! ```rust
//! use openvpn_management::EventManager;
//! # use std::net::TcpListener;
//! # use std::io::{BufRead, BufReader, Write};
//! # use std::thread;
//! # let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\nEND";
//! # let listener = TcpListener::bind("127.0.0.1:5555".to_string()).unwrap();
//! # thread::spawn(move || {
//! #    for client_stream in listener.incoming() {
//! #        let mut stream = client_stream.unwrap();
//! #        let mut reader = BufReader::new(&stream);
//! #        let mut output = String::new();
//! #        reader.read_line(&mut output).unwrap();
//! #        assert_eq!("status\n".to_string(), output);
//! #        stream.write(server_response.as_bytes()).unwrap();
//! #        break
//! #    }
//! # });
//! // build the client:
//! let mut event_manager = openvpn_management::CommandManagerBuilder::new()
//!     .management_url("localhost:5555")
//!     .build()
//!     .unwrap();
//! // get the current status:
//! let status = event_manager
//!     .get_status()
//!     .unwrap();
//! // get client information:
//! let clients = status.clients();
//! ```
mod client;
mod error;

pub use crate::client::Client;
pub use crate::error::{OpenvpnError, OpenvpnResult as Result};
use chrono::prelude::{DateTime, TimeZone, Utc};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

const DEFAULT_MANAGEMENT_URL: &str = "localhost:5555";
const ENDING: &str = "END";
const START_LINE: &str = "CLIENT_LIST";
const HEADER_START_LINE: &str = "HEADER\tCLIENT_LIST";
const UNDEF: &str = "UNDEF";

#[derive(Clone, Debug)]
pub struct Status {
    clients: Vec<Client>,
}

impl Status {
    pub fn new(clients: Vec<Client>) -> Status {
        Status { clients }
    }

    pub fn clients(&self) -> &[Client] {
        &self.clients
    }
}

pub struct CommandManager {
    management_address: SocketAddr,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

pub trait EventManager {
    fn get_status(&mut self) -> Result<Status>;
}

impl EventManager for CommandManager {
    /// Creates a new TCP connection to the management interface and sends a status request.
    /// The response is then parsed into the status response with the client information. This
    /// can be used by applications which are polling the management interface for status updates
    fn get_status(&mut self) -> Result<Status> {
        let mut stream = match self.connect_timeout {
            Some(ct) => TcpStream::connect_timeout(&self.management_address, ct)?,
            None => TcpStream::connect(&self.management_address)?,
        };
        stream.set_read_timeout(self.read_timeout)?;
        stream.write_all(b"status\n")?;
        let mut reader = BufReader::new(&stream);

        let mut output = String::new();
        while !output.trim().ends_with(ENDING) {
            reader.read_line(&mut output)?;
        }

        let clients = parse_status_output(output)?;
        Ok(Status { clients })
    }
}

pub struct CommandManagerBuilder {
    management_url: String,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl CommandManagerBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn management_url(&mut self, management_url: &str) -> &mut CommandManagerBuilder {
        self.management_url = management_url.to_owned();
        self
    }

    pub fn connect_timeout(
        &mut self,
        connect_timeout: Option<Duration>,
    ) -> &mut CommandManagerBuilder {
        self.connect_timeout = connect_timeout;
        self
    }

    pub fn read_timeout(&mut self, read_timeout: Option<Duration>) -> &mut CommandManagerBuilder {
        self.read_timeout = read_timeout;
        self
    }

    pub fn build(&mut self) -> Result<CommandManager> {
        let mut addrs_iter = self.management_url.to_socket_addrs()?;

        let management_address: SocketAddr = match addrs_iter.next() {
            Some(a) => a,
            None => {
                return Err(OpenvpnError::MissingURLInput(
                    self.management_url.to_owned(),
                ));
            }
        };

        Ok(CommandManager {
            management_address,
            read_timeout: self.read_timeout,
            connect_timeout: self.connect_timeout,
        })
    }
}

impl Default for CommandManagerBuilder {
    fn default() -> Self {
        CommandManagerBuilder {
            management_url: DEFAULT_MANAGEMENT_URL.to_owned(),
            connect_timeout: None,
            read_timeout: None,
        }
    }
}

fn parse_status_output(output: String) -> Result<Vec<Client>> {
    let split = output.split('\n');
    let mut clients = Vec::new();
    let mut has_client_list = false;
    for s in split {
        let line = String::from(s);
        if line.starts_with(HEADER_START_LINE) {
            has_client_list = true;
        }
        if line.starts_with(START_LINE) {
            let client = parse_client(&line)?;
            if client.name() != UNDEF {
                clients.push(client);
            }
        }
    }
    if has_client_list {
        return Ok(clients);
    }
    Err(OpenvpnError::MalformedResponse(output))
}

fn parse_client(raw_client: &str) -> Result<Client> {
    let vec: Vec<_> = raw_client.split('\t').collect();
    if vec.len() < 9 {
        return Err(OpenvpnError::MalformedResponse(raw_client.to_string()));
    }
    let name = vec[1];
    let address = vec[2]
        .split(':')
        .next()
        .ok_or_else(|| OpenvpnError::MalformedResponse(raw_client.to_string()))?;
    let timestamp: i64 = vec[8].parse()?;
    let bytes_received: f64 = vec[5].parse()?;
    let bytes_sent: f64 = vec[6].parse()?;
    Ok(Client::new(
        String::from(name),
        String::from(address),
        get_utc_start_time(timestamp),
        bytes_received,
        bytes_sent,
    ))
}

fn get_utc_start_time(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp(timestamp, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_management_url_parsed_correctly() {
        let result = CommandManagerBuilder::new()
            .management_url("192.168.0.1:12345")
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_management_localhost_url_parsed_correctly() {
        let result = CommandManagerBuilder::new()
            .management_url("localhost:12345")
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_malformed_management_url() {
        let malformed_url = "foo:12345";

        let result = CommandManagerBuilder::new()
            .management_url(malformed_url)
            .build();

        assert!(result.is_err());
        let expected_error = match result {
            Err(OpenvpnError::Io(ref e)) => e,
            _ => panic!("expected io error"),
        };

        assert_eq!(io::ErrorKind::Other, expected_error.kind());
    }

    #[test]
    fn test_missing_management_url() {
        let malformed_url = "";

        let result = CommandManagerBuilder::new()
            .management_url(malformed_url)
            .build();

        assert!(result.is_err());
        let expected_error = match result {
            Err(OpenvpnError::Io(ref e)) => e,
            _ => panic!("expected io error"),
        };

        assert_eq!(io::ErrorKind::InvalidInput, expected_error.kind());
    }
}
