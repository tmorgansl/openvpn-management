use chrono::prelude::{DateTime, Local};

#[derive(Clone, Debug, PartialEq)]
/// Contains useful information on a client which is connected to the openvpn server
pub struct Client {
    name: String,
    ip_address: String,
    connected_since: DateTime<Local>,
    bytes_received: f64,
    bytes_sent: f64,
}

impl Client {
    pub fn new(
        name: String,
        ip_address: String,
        connected_since: DateTime<Local>,
        bytes_received: f64,
        bytes_sent: f64,
    ) -> Client {
        Client {
            name,
            ip_address,
            connected_since,
            bytes_received,
            bytes_sent,
        }
    }

    /// Common Name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Remote IP address
    pub fn ip_address(&self) -> &str {
        &self.ip_address
    }

    /// Date time they connected at
    pub fn connected_since(&self) -> &DateTime<Local> {
        &self.connected_since
    }

    /// Bytes received from the client
    pub fn bytes_received(&self) -> f64 {
        self.bytes_received
    }

    /// Bytes sent to remote servers
    pub fn bytes_sent(&self) -> f64 {
        self.bytes_sent
    }
}
