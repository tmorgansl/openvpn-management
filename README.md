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
    .management_url("localhost:5555")
    .build();
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

