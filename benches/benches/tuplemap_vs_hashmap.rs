#![feature(test)] extern crate test;

#[derive(Default)]
struct AsIsHasher(u64);
impl std::hash::Hasher for AsIsHasher {
    #[cold] #[inline(never)] fn write(&mut self, _: &[u8]) {
        unsafe {std::hint::unreachable_unchecked()}
    }
    #[inline(always)] fn write_u64(&mut self, type_id_value: u64) {
        self.0 = type_id_value
    }
    #[inline(always)] fn finish(&self) -> u64 {
        self.0
    }
}
            
macro_rules! tuplemap_vs_hashmap {
    ($name:ident : size = $size:literal, iter = $search_iteration:literal) => {
        #[allow(non_snake_case)]
        mod $name {
            use super::*;

            #[bench]
            fn create_tuplemap(b: &mut test::Bencher) {
                use ohkami_lib::map::TupleMap;
                use rand::prelude::*;
            
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(314159265358979);
                let keys: [u64; $size] = std::array::from_fn(|_| rng.r#gen());
            
                b.iter(|| -> TupleMap<u64, Box<String>> {
                    TupleMap::<u64, Box<String>>::from_iter(
                        keys.map(|k| (k, Box::new(k.to_string())))
                    )
                })
            }

            #[bench]
            fn search_tuplemap(b: &mut test::Bencher) {
                use ohkami_lib::map::TupleMap;
                use rand::prelude::*;
            
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(314159265358979);
                let keys: [u64; $size] = std::array::from_fn(|_| rng.r#gen());

                let map = TupleMap::<u64, Box<String>>::from_iter(
                    keys.map(|k| (k, Box::new(k.to_string())))
                );
            
                b.iter(|| -> String {
                    (0..$search_iteration).fold(String::new(), |mut a, _| {
                        let i = rng.gen_range(0..$size * 2) as u64;
                        if let Some(v) = map.get(&i) {
                            a.push_str(v)
                        }
                        a
                    })
                })
            }
            
            #[bench]
            fn create_hashmap(b: &mut test::Bencher) {
                use std::{collections::HashMap, hash::BuildHasherDefault};
                use rand::prelude::*;
                
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(314159265358979);
                let keys: [u64; $size] = std::array::from_fn(|_| rng.r#gen());            
            
                b.iter(|| -> HashMap::<u64, Box<String>, BuildHasherDefault<AsIsHasher>> {
                    HashMap::<u64, Box<String>, BuildHasherDefault<AsIsHasher>>::from_iter(
                        keys.map(|k| (k, Box::new(k.to_string())))
                    )
                })
            }
            
            #[bench]
            fn search_hashmap(b: &mut test::Bencher) {
                use std::{collections::HashMap, hash::BuildHasherDefault};
                use rand::prelude::*;
                
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(314159265358979);
                let keys: [u64; $size] = std::array::from_fn(|_| rng.r#gen());

                let map = HashMap::<u64, Box<String>, BuildHasherDefault<AsIsHasher>>::from_iter(
                    keys.map(|k| (k, Box::new(k.to_string())))
                );            
            
                b.iter(|| -> String {
                    (0..$search_iteration).fold(String::new(), |mut a, _| {
                        let i = rng.gen_range(0..$size * 2) as u64;
                        if let Some(v) = map.get(&i) {
                            a.push_str(v)
                        }
                        a
                    })
                })
            }
        }
    }
}

tuplemap_vs_hashmap!(size1_search2__ : size = 1, iter = 2  );
tuplemap_vs_hashmap!(size1_search10_ : size = 1, iter = 10 );
tuplemap_vs_hashmap!(size1_search100 : size = 1, iter = 100);

tuplemap_vs_hashmap!(size2_search2__ : size = 2, iter = 2  );
tuplemap_vs_hashmap!(size2_search10_ : size = 2, iter = 10 );
tuplemap_vs_hashmap!(size2_search100 : size = 2, iter = 100);

tuplemap_vs_hashmap!(size4_search2__ : size = 4, iter = 2  );
tuplemap_vs_hashmap!(size4_search10_ : size = 4, iter = 10 );
tuplemap_vs_hashmap!(size4_search100 : size = 4, iter = 100);

tuplemap_vs_hashmap!(size8_search2__ : size = 8, iter = 2  );
tuplemap_vs_hashmap!(size8_search10_ : size = 8, iter = 10 );
tuplemap_vs_hashmap!(size8_search100 : size = 8, iter = 100);
