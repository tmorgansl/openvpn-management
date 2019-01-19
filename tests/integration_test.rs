extern crate openvpn_management;
use chrono::prelude::{DateTime, TimeZone, Utc};
use openvpn_management::{Client, EventManager, OpenvpnError, Status};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const READ_TIMEOUT: Duration = Duration::from_millis(1000);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(2000);

fn setup_tcp_server(
    port: u16,
    response: &'static str,
    response_sleep: Option<Duration>,
) -> thread::JoinHandle<()> {
    let mut connection_string = "localhost:".to_string();
    connection_string.push_str(&port.to_string());
    let listener = TcpListener::bind(connection_string).unwrap();
    thread::spawn(move || {
        let mut stream = listener.accept().unwrap().0;
        let mut reader = BufReader::new(&stream);
        let mut output = String::new();
        reader.read_line(&mut output).unwrap();
        assert_eq!("status\n".to_string(), output);
        if let Some(r) = response_sleep {
            thread::sleep(r);
        }
        stream.write_all(response.as_bytes()).unwrap();
    })
}

fn new_mock_client(
    name: &'static str,
    ip_address: &'static str,
    epoch_seconds: i64,
    bytes_received: f64,
    bytes_sent: f64,
) -> Client {
    let datetime: DateTime<Utc> = Utc.timestamp(epoch_seconds, 0);
    Client::new(
        name.to_string(),
        ip_address.to_string(),
        datetime,
        bytes_received,
        bytes_sent,
    )
}

fn new_mock_status(title: &'static str, epoch_seconds: i64, clients: Vec<Client>) -> Status {
    let datetime: DateTime<Utc> = Utc.timestamp(epoch_seconds, 0);
    Status::new(String::from(title), datetime, clients)
}

#[test]
fn test_no_client_list_in_response() {
    let server_response = "no client string END";
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let response = match status_response {
        Err(OpenvpnError::MalformedResponse(e)) => e,
        _ => panic!("was expecting malformed response"),
    };

    assert_eq!(server_response, response);
}

#[test]
fn test_empty_clients_in_response() {
    let server_response =
        "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nEND";
    let expected_status = new_mock_status("test-title", 1547913893, Vec::new());
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(expected_status, status);
}

#[test]
fn test_client_details_too_short_in_response() {
    let server_response = "\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST bad\tclient\tinformation\r\nEND";
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let response = match status_response {
        Err(OpenvpnError::MalformedResponse(e)) => e,
        _ => panic!("was expecting malformed response"),
    };

    assert_eq!("CLIENT_LIST bad\tclient\tinformation", response);
}

#[test]
fn test_client_correct_details_in_response() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\r\nEND";
    let expected_client = new_mock_client("test-client", "127.0.0.1", 1_546_277_714, 100.0, 200.0);
    let expected_status = new_mock_status("test-title", 1547913893, vec![expected_client; 1]);
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(expected_status, status);
}

#[test]
fn test_multiple_clients_details() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\r\nCLIENT_LIST\ttest-client2\t192.168.0.3:12345\t10.8.0.3\t\t300\t400\tdate-string\t1546277715\r\nEND";
    let expected_clients = vec![new_mock_client(
        "test-client",
        "127.0.0.1",
        1_546_277_714,
        100.0,
        200.0,
    ), new_mock_client(
        "test-client2",
        "192.168.0.3",
        1_546_277_715,
        300.0,
        400.0,
    )];
    let expected_status = new_mock_status("test-title", 1547913893, expected_clients);
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(expected_status, status);
}

#[test]
fn test_parse_error_in_client_response_bytes_received() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\tNAN_STRING\t200\tdate-string\t1546277714\r\nEND";
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseFloat(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse float");
}

#[test]
fn test_parse_error_in_client_response_bytes_sent() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\tNAN_STRING\tdate-string\t1546277714\r\nEND";
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseFloat(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse float");
}

#[test]
fn test_parse_error_in_client_response_timestamp() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\tNAN_DATE_TIME\r\nEND";
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseInt(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse int");
}

#[test]
fn test_io_error_on_missing_server() {
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::Io(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected io error");
}

#[test]
fn test_client_correct_details_within_read_timeout() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\r\nEND";
    let expected_client = new_mock_client("test-client", "127.0.0.1", 1_546_277_714, 100.0, 200.0);
    let expected_status = new_mock_status("test-title", 1547913893, vec![expected_client; 1]);
    let read_latency = READ_TIMEOUT - Duration::from_millis(100);
    let handle = setup_tcp_server(5555, server_response, Some(read_latency));
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .read_timeout(Some(READ_TIMEOUT))
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(expected_status, status);
}

#[test]
fn test_client_error_with_slow_server_response() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\r\nEND";
    let read_latency = READ_TIMEOUT + Duration::from_millis(100);
    let handle = setup_tcp_server(5555, server_response, Some(read_latency));
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .read_timeout(Some(READ_TIMEOUT))
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::Io(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected io error");
}

#[test]
fn test_client_correct_details_within_connect_timeout() {
    let server_response = "TITLE\ttest-title\r\nTIME\ttimestamp\t1547913893\r\nHEADER\tCLIENT_LIST\r\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\r\nEND";
    let expected_client = new_mock_client("test-client", "127.0.0.1", 1_546_277_714, 100.0, 200.0);
    let expected_status = new_mock_status("test-title", 1547913893, vec![expected_client; 1]);
    let handle = setup_tcp_server(5555, server_response, None);
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .connect_timeout(Some(CONNECT_TIMEOUT))
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    handle.join().unwrap();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(expected_status, status);
}

#[test]
fn test_client_error_slow_connection() {
    let mut api = openvpn_management::CommandManagerBuilder::new()
        .management_url("10.255.255.1:5555")
        .connect_timeout(Some(CONNECT_TIMEOUT))
        .build()
        .expect("api build successfully");
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::Io(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected io error");
}
