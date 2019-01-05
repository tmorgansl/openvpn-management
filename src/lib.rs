/*!
openvpn-management is a wrapper to the [openvpn management interface](https://openvpn.net/community-resources/management-interface/).
# Install:
The crate is called `openvpn-management` and you can depend on it via cargo:
```ini
[dependencies]
openvpn-management = "*"
```
# Features:
- Getting all connected client information
# Basic usage:
```rust
use openvpn_management::EventManager;
# use std::net::TcpListener;
# use std::io::{BufRead, BufReader, Write};
# use std::thread;
# let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\nEND";
# let listener = TcpListener::bind("localhost:5555".to_string()).unwrap();
# thread::spawn(move || {
#    for client_stream in listener.incoming() {
#        let mut stream = client_stream.unwrap();
#        let mut reader = BufReader::new(&stream);
#        let mut output = String::new();
#        reader.read_line(&mut output).unwrap();
#        assert_eq!("status\n".to_string(), output);
#        stream.write(server_response.as_bytes()).unwrap();
#        break
#    }
# });
// build the client:
let mut event_manager = openvpn_management::CommandManagerBuilder::new()
    .management_url("localhost:5555")
    .build();
// get the current status:
let status = event_manager
    .get_status()
    .unwrap();
// get client information:
let clients = status.clients();
```
!*/

mod client;
mod error;

pub use crate::client::Client;
pub use crate::error::{OpenvpnError, OpenvpnResult as Result};
use chrono::prelude::{DateTime, Local, TimeZone};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

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
    pub fn clients(&self) -> &Vec<Client> {
        &self.clients
    }
}

pub struct CommandManager {
    management_url: String,
}

pub trait EventManager {
    fn get_status(&mut self) -> Result<Status>;
}

impl EventManager for CommandManager {
    /*!
    Creates a new TCP connection to the management interface and sends a status request.
    The response is then parsed into the status response with the client information. This
    can be used by applications which are polling the management interface for status updates
    */
    fn get_status(&mut self) -> Result<Status> {
        let mut stream = TcpStream::connect(self.management_url.to_owned())?;
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
}

impl CommandManagerBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn management_url(&mut self, url: &str) -> &mut CommandManagerBuilder {
        self.management_url = url.to_owned();
        self
    }

    pub fn build(&mut self) -> CommandManager {
        CommandManager {
            management_url: self.management_url.to_owned(),
        }
    }
}

impl Default for CommandManagerBuilder {
    fn default() -> Self {
        CommandManagerBuilder {
            management_url: DEFAULT_MANAGEMENT_URL.to_owned(),
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
    let split = raw_client.split('\t');
    let vec = split.collect::<Vec<&str>>();
    if vec.len() < 9 {
        return Err(OpenvpnError::MalformedResponse(raw_client.to_string()));
    }
    let name = vec[1];
    let address = match vec[2].split(':').next() {
        Some(a) => a,
        None => return Err(OpenvpnError::MalformedResponse(raw_client.to_string())),
    };
    let timestamp = vec[8].parse::<i64>()?;
    let bytes_received = vec[5].parse::<f64>()?;
    let bytes_sent = vec[6].parse::<f64>()?;
    Ok(Client::new(
        String::from(name),
        String::from(address),
        get_local_start_time(timestamp),
        bytes_received,
        bytes_sent,
    ))
}

fn get_local_start_time(timestamp: i64) -> DateTime<Local> {
    let datetime: DateTime<Local> = Local.timestamp(timestamp, 0);
    datetime
}
