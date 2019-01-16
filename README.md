Openvpn Management
=========================
[![Build Status](https://travis-ci.com/tmorgansl/openvpn-management.svg?branch=master)](https://travis-ci.com/tmorgansl/openvpn-management)
[![License](https://img.shields.io/github/license/tmorgansl/openvpn-management.svg)]()
[![Crates.io](https://img.shields.io/crates/v/openvpn-management.svg)](https://crates.io/crates/openvpn-management)
[![Docs.rs](https://docs.rs/openvpn-management/badge.svg)](https://docs.rs/openvpn-management)

openvpn-management is a wrapper to the [openvpn management interface](https://openvpn.net/community-resources/management-interface/) for rust applications.
# Install:
The crate is called `openvpn-management` and you can depend on it via cargo:
```
[dependencies]
openvpn-management = "*"
```
### Features:
- Getting all connected client information

### Basic usage:

```
// build the client:
let mut event_manager = openvpn_management::CommandManagerBuilder::new()
    .management_url("127.0.0.1:5555")
    .build()
    .unwrap();
// get the current status:
let status = event_manager
    .get_status()
    .unwrap();
// get client information:
let clients = status.clients();
```

### Roadmap

This library is in the early stages of development and as such only supports extracting client information using the `status` command. PRs with additional functionality would always be welcome.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

