pub(crate) fn tcp_address(address_str: &'static str) -> String {
    if address_str.starts_with(":") {
        "127.0.0.1".to_owned() + address_str
    } else if address_str.starts_with("localhost") {
        address_str.replace("localhost", "127.0.0.1")
    } else {
        address_str.to_owned()
    }
}