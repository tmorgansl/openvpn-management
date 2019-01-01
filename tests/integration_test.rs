extern crate openvpn_management;
use std::net::TcpListener;
use std::io::{BufRead, BufReader, Write};
use std::thread;
use openvpn_management::{EventManager, OpenvpnError, Client};
use chrono::prelude::{DateTime, Local, TimeZone};

fn setup_tcp_server(port: u16, response: &'static str) -> thread::JoinHandle<()> {
    let mut connection_string = "localhost:".to_string();
    connection_string.push_str(&mut port.to_string());
    let listener = TcpListener::bind(connection_string).unwrap();
    thread::spawn(move || {
        for client_stream in listener.incoming() {
            let mut stream = client_stream.unwrap();
            let mut reader = BufReader::new(&stream);
            let mut output = String::new();
            reader.read_line(&mut output).unwrap();
            assert_eq!("status\n".to_string(), output);
            stream.write(response.as_bytes()).unwrap();
            break
        }
    })
}

fn new_mock_client(name: &'static str, ip_address: &'static str, epoch_seconds: i64, bytes_received: f64, bytes_sent: f64) -> Client {
    let datetime: DateTime<Local> = Local.timestamp(epoch_seconds, 0);
    Client::new(name.to_string(), ip_address.to_string(), datetime, bytes_received, bytes_sent)
}

#[test]
fn test_no_client_list_in_response() {
    let server_response = "no client string END";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let response = match status_response {
        Err(OpenvpnError::MalformedResponse(e)) => e,
        _ => panic!("was expecting malformed response"),
    };

    assert_eq!(server_response, response);
    handle.join().unwrap();
}

#[test]
fn test_empty_clients_in_response() {
    let server_response = "\nHEADER\tCLIENT_LIST\nEND";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(0, status.clients().len());
    handle.join().unwrap();
}

#[test]
fn test_client_details_too_short_in_response() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST bad\tclient\tinformation\nEND";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let response = match status_response {
        Err(OpenvpnError::MalformedResponse(e)) => e,
        _ => panic!("was expecting malformed response"),
    };

    assert_eq!("CLIENT_LIST bad\tclient\tinformation", response);
    handle.join().unwrap();
}

#[test]
fn test_client_correct_details_in_response() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\nEND";
    let expected_client = new_mock_client("test-client", "127.0.0.1", 1546277714, 100.0, 200.0);
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(1, status.clients().len());
    let client = &status.clients()[0];
    assert_eq!(&expected_client, client);
    handle.join().unwrap();
}

#[test]
fn test_multiple_clients_details() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\t1546277714\nCLIENT_LIST\ttest-client2\t192.168.0.3:12345\t10.8.0.3\t\t300\t400\tdate-string\t1546277715\nEND";
    let mut expected_clients = Vec::new();
    expected_clients.push(new_mock_client("test-client", "127.0.0.1", 1546277714, 100.0, 200.0));
    expected_clients.push(new_mock_client("test-client2", "192.168.0.3", 1546277715, 300.0, 400.0));

    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_ok());
    let status = status_response.unwrap();
    assert_eq!(&expected_clients, status.clients());
    handle.join().unwrap();
}


#[test]
fn test_parse_error_in_client_response_bytes_received() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\tNAN_STRING\t200\tdate-string\t1546277714\nEND";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseFloat(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse float");
    handle.join().unwrap();
}

#[test]
fn test_parse_error_in_client_response_bytes_sent() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\tNAN_STRING\tdate-string\t1546277714\nEND";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseFloat(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse float");
    handle.join().unwrap();
}

#[test]
fn test_parse_error_in_client_response_timestamp() {
    let server_response = "\nHEADER\tCLIENT_LIST\nCLIENT_LIST\ttest-client\t127.0.0.1:12345\t10.8.0.2\t\t100\t200\tdate-string\tNAN_DATE_TIME\nEND";
    let handle = setup_tcp_server(5555, server_response);
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::ParseInt(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected unable to parse int");
    handle.join().unwrap();
}

#[test]
fn test_io_error_on_missing_server() {
    let mut api = openvpn_management::CommandManagerBuilder::new().build();
    let status_response = api.get_status();
    assert!(status_response.is_err());
    let expected_error = match status_response {
        Err(OpenvpnError::Io(_)) => true,
        _ => false,
    };

    assert!(expected_error, "expected io error");
}