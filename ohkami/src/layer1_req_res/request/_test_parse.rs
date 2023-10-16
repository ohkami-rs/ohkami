use super::{Request, METADATA_SIZE};
use crate::{__rt__::{test}, Method};

macro_rules! assert_eq {
    ($case:expr, $expected:expr) => {
        std::assert_eq!(Request::new(&mut $case.as_bytes()).await, $expected);
    };
}

fn metadataize(input: &str) -> [u8; METADATA_SIZE] {
    let mut metadata = [0; METADATA_SIZE];
    metadata.copy_from_slice(input.as_bytes());
    metadata
}

#[test] async fn test_parse_request() {
    let case = "";
    assert_eq!(
        case,
        Request {
            _metadata: metadataize(case),
            payload: None,
            method: Method::GET,
            path: ,
        }
    );
}
