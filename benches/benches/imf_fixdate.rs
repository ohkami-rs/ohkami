#![feature(test)]
extern crate test;

use ohkami_lib::time::UTCDateTime;
use ohkami::util::unix_timestamp;


#[bench] fn format_imf_fixdate(b: &mut test::Bencher) {
    b.iter(|| {
        UTCDateTime::from_unix_timestamp(unix_timestamp())
            .into_imf_fixdate();
    })
}
