#![feature(test)]
extern crate test;

use ohkami_lib::time::UTCDateTime;
use ohkami::utils::unix_timestamp;


#[bench] fn format_imf_fixdate(b: &mut test::Bencher) {
    b.iter(|| {
        UTCDateTime::from_duration_since_unix_epoch(
            std::time::Duration::from_secs(unix_timestamp())
        ).into_imf_fixdate();
    })
}
