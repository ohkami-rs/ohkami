use super::parse::{Multipart, Part, TextOrFiles};
use super::File;


#[test] fn parse_single() {
    const BOUNDARY: &str = "AaB03x";

    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --{BOUNDARY}--");

    assert_eq!(
        Multipart::parse(case.as_bytes()).unwrap(),
        Multipart(vec![
            Part::Text { name: "user-name", text: "Joe Blow" },
        ])
    );
}
