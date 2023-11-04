pub const CHANK: usize = 64;
pub const SIZE:  usize = 20; // bytes; 160 bits

pub struct Sha1 {
    h:   [u32; 5],
    x:   [u8; CHANK],
    nx:  usize,
    len: u64,
}

const K0: u32 = 0x5A827999;
const K1: u32 = 0x6ED9EBA1;
const K2: u32 = 0x8F1BBCDC;
const K3: u32 = 0xCA62C1D6;

// https://github.com/golang/go/blob/master/src/crypto/sha1/sha1.go
impl Sha1 {
    pub fn new() -> Self {
        Self {
            h:   [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            x:   [0; CHANK],
            nx:  0,
            len: 0,
        }
    }

    pub fn write(&mut self, mut p: &[u8]) {
        self.len += p.len() as u64;
        if self.nx > 0 {
            let n = (CHANK - self.nx).min(p.len());
            self.x[self.nx..(self.nx + n)].copy_from_slice(&p[..n]);
            self.nx += n;
            if self.nx == CHANK {
                self.block(&self.x.clone());
                self.nx = 0;
            }
            p = &p[n..]
        }
        if p.len() >= CHANK {
            let n = p.len() & (!(CHANK - 1));
            self.block(&p[..n]);
            p = &p[n..]
        }
        if p.len() > 0 {
            self.nx = self.x.len().min(p.len());
            self.x.copy_from_slice(p);
        }
    }

    pub fn sum(mut self) -> [u8; SIZE] {
        let mut len = self.len;

        let mut tmp = [0; 64 + 8];
        tmp[0] = 0x80;
        let t = if len%64 < 56 {
            56 - len%64
        } else {
            64 + 56 - len%64
        };

        len <<= 3;
        //let padlen = &mut tmp[..(t as usize + 8)];
        //padlen[(t as usize)..].copy_from_slice(&len.to_be_bytes());
        //self.write(padlen);
        tmp[(t as usize)..(t as usize + 8)].copy_from_slice(&len.to_be_bytes());
        self.write(&tmp[..(t as usize + 8)]);

        #[cfg(debug_assertions)] assert_eq!(self.nx, 0);

        let mut digest = [0; SIZE];
        digest[0..  4].copy_from_slice(&self.h[0].to_be_bytes());
        digest[4..  8].copy_from_slice(&self.h[1].to_be_bytes());
        digest[8.. 12].copy_from_slice(&self.h[2].to_be_bytes());
        digest[12..16].copy_from_slice(&self.h[3].to_be_bytes());
        digest[16..  ].copy_from_slice(&self.h[4].to_be_bytes());
        digest
    }
}

// https://github.com/golang/go/blob/master/src/crypto/sha1/sha1block.go
impl Sha1 {
    fn block(&mut self, mut p: &[u8]) {
        let mut w = [0u32; 16];

        let (mut h0, mut h1, mut h2, mut h3, mut h4) = (self.h[0], self.h[1], self.h[2], self.h[3], self.h[4]);
        while p.len() >= CHANK {
            for i in 0..16 {
                let j = i * 4;
                w[i] = (p[j] as u32) << 24 | (p[j+1] as u32) << 16 | (p[j+2] as u32) << 8 | (p[j+3] as u32);
            }

            let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);

            for i in 0..16 {
                let f = (b & c) | ((!b) & d);
                let t = dbg!(a.rotate_left(5)) + dbg!(f) + e + w[i&0xf] + K0;
                (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d)
            }
            for i in 16..20 {
                let tmp = w[(i-3)&0xf] ^ w[(i-8)&0xf] ^ w[(i-14)&0xf] ^ w[(i)&0xf];
                w[i&0xf] = tmp.rotate_left(1);

                let f = (b & c) | ((!b) & d);
			    let t = a.rotate_left(5) + f + e + w[i & 0xf] + K0;
			    (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d)
            }
            for i in 20..40 {
                let tmp = w[(i-3)&0xf] ^ w[(i-8)&0xf] ^ w[(i-14)&0xf] ^ w[(i)&0xf];
			    w[i&0xf] = tmp.rotate_left(1);

			    let f = b ^ c ^ d;
			    let t = a.rotate_left(5) + f + e + w[i&0xf] + K1;
			    (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d);
            }
            for i in 40..60 {
                let tmp = w[(i-3)&0xf] ^ w[(i-8)&0xf] ^ w[(i-14)&0xf] ^ w[(i)&0xf];
			    w[i&0xf] = tmp.rotate_left(1);

			    let f = ((b | c) & d) | (b & c);
			    let t = a.rotate_left(5) + f + e + w[i&0xf] + K2;
			    (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d);
            }
            for i in 60..80 {
                let tmp = w[(i-3)&0xf] ^ w[(i-8)&0xf] ^ w[(i-14)&0xf] ^ w[(i)&0xf];
			    w[i&0xf] = tmp.rotate_left(1);

			    let f = b ^ c ^ d;
			    let t = a.rotate_left(5) + f + e + w[i&0xf] + K3;
			    (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d);
            }

            h0 += a;
            h1 += b;
            h2 += c;
            h3 += d;
            h4 += e;

            p = &p[CHANK..]
        }

        (self.h[0], self.h[1], self.h[2], self.h[3], self.h[4]) = (h0, h1, h2, h3, h4)
    }    
}
