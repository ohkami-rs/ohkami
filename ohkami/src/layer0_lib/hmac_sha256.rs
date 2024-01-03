use std::borrow::Cow;


const CHUNK:      usize = 64;
const SIZE:       usize = 32/* 256 bits */;
const BLOCK_SIZE: usize = 64;

const MAGIC_224: &'static str = "sha\x02";
const MAGIC_256: &'static str = "sha\x03";
const MARSHALIZED_SIZE: usize = MAGIC_256.len() + 8*4 + CHUNK + 8;


#[allow(non_camel_case_types)]
pub struct HMAC_SHA256 {
    opad:      Vec<u8>,
    ipad:      Vec<u8>,
    outer:     SHA256,
    inner:     SHA256,
}

impl HMAC_SHA256 {
    pub fn new(secret_key: impl AsRef<[u8]>) -> Self {
        let mut secret_key = Cow::<'_, [u8]>::Borrowed(secret_key.as_ref());

        let mut this = HMAC_SHA256 {
            opad:      vec![0; BLOCK_SIZE],
            ipad:      vec![0; BLOCK_SIZE],
            outer:     SHA256::new(),
            inner:     SHA256::new(),
        };

        if secret_key.len() > BLOCK_SIZE {
            this.outer.write(&secret_key);
            secret_key = Cow::Owned(this.outer.clone().sum().to_vec());
        }
        this.ipad.copy_from_slice(&secret_key);
        this.opad.copy_from_slice(&secret_key);
        for p in &mut this.ipad {
            *p ^= 0x36;
        }
        for p in &mut this.opad {
            *p ^= 0x5c;
        }
        this.inner.write(&this.ipad);

        this
    }

    pub fn write(&mut self, p: &[u8]) {
        self.inner.write(p)
    }

    pub fn sum(self) -> [u8; SIZE] {
        let Self { opad, ipad:_, mut outer, inner } = self;

        let in_sum = inner.sum();

        outer.reset();
        outer.write(&opad);

        outer.write(&in_sum);
        outer.sum()
    }
}


#[derive(Clone)]
pub struct SHA256 {
    h:   [u32; 8],
    x:   [u8; CHUNK],
    nx:  usize,
    len: usize,
}

impl SHA256 {
    pub fn new() -> Self {
        Self {
            h:   [
                0x6A09E667,
	            0xBB67AE85,
	            0x3C6EF372,
	            0xA54FF53A,
	            0x510E527F,
	            0x9B05688C,
	            0x1F83D9AB,
	            0x5BE0CD19,
            ],
            x:   [0; CHUNK],
            nx:  0,
            len: 0,
        }
    }

    pub fn write(&mut self, mut p: &[u8]) {
        let nn = p.len();

        self.len += nn;

        if self.nx > 0 {
            let n = usize::min(self.x.len() - self.nx, p.len());
            self.x[self.nx..(self.nx + n)].copy_from_slice(&p[..n]);
            if self.nx == CHUNK {
                self.block(&self.x.clone());
                self.nx = 0;
            }
            p = &p[n..];
        }
        if p.len() >= CHUNK {
            let n = p.len() & (!(CHUNK - 1));
            self.block(&p[..n]);
            p = &p[n..];
        }
        if p.len() > 0 {
            let n = usize::min(self.x.len(), p.len());
            self.x[..n].copy_from_slice(&p[..n]);
            self.nx = n;
        }
    }

    pub fn sum(mut self) -> [u8; SIZE] {
        let mut len = self.len;

        let mut tmp = [u8::default(); 64+8];
        tmp[0] = 0x80;

        let t = if len%64 < 56 {
            56 - len%64
        } else {
            64 + 56 - len%64
        };

        len <<= 3; // Length in bits
        tmp[t..t+8].copy_from_slice(&(len as u64).to_be_bytes());
        self.write(&tmp[..t+8]);

        debug_assert!(self.nx == 0, "`self.nx` is not 0");

        let mut digest = [u8::default(); 32];
        digest[0.. 4 ].copy_from_slice(&self.h[0].to_be_bytes());
        digest[4.. 8 ].copy_from_slice(&self.h[1].to_be_bytes());
        digest[8.. 12].copy_from_slice(&self.h[2].to_be_bytes());
        digest[12..16].copy_from_slice(&self.h[3].to_be_bytes());
        digest[16..20].copy_from_slice(&self.h[4].to_be_bytes());
        digest[20..24].copy_from_slice(&self.h[5].to_be_bytes());
        digest[24..28].copy_from_slice(&self.h[6].to_be_bytes());
        digest[28..32].copy_from_slice(&self.h[7].to_be_bytes());
        digest
    }
}

impl SHA256 {
    fn reset(&mut self) {
        *self = Self {
            x: self.x,
            ..Self::new()
        }
    }
}

impl SHA256 {
    fn block(&mut self, mut p: &[u8]) {
        const K: [u32; 64] = [
            0x428a2f98,
	        0x71374491,
	        0xb5c0fbcf,
	        0xe9b5dba5,
	        0x3956c25b,
	        0x59f111f1,
	        0x923f82a4,
	        0xab1c5ed5,
	        0xd807aa98,
	        0x12835b01,
	        0x243185be,
	        0x550c7dc3,
	        0x72be5d74,
	        0x80deb1fe,
	        0x9bdc06a7,
	        0xc19bf174,
	        0xe49b69c1,
	        0xefbe4786,
	        0x0fc19dc6,
	        0x240ca1cc,
	        0x2de92c6f,
	        0x4a7484aa,
	        0x5cb0a9dc,
	        0x76f988da,
	        0x983e5152,
	        0xa831c66d,
	        0xb00327c8,
	        0xbf597fc7,
	        0xc6e00bf3,
	        0xd5a79147,
	        0x06ca6351,
	        0x14292967,
	        0x27b70a85,
	        0x2e1b2138,
	        0x4d2c6dfc,
	        0x53380d13,
	        0x650a7354,
	        0x766a0abb,
	        0x81c2c92e,
	        0x92722c85,
	        0xa2bfe8a1,
	        0xa81a664b,
	        0xc24b8b70,
	        0xc76c51a3,
	        0xd192e819,
	        0xd6990624,
	        0xf40e3585,
	        0x106aa070,
	        0x19a4c116,
	        0x1e376c08,
	        0x2748774c,
	        0x34b0bcb5,
	        0x391c0cb3,
	        0x4ed8aa4a,
	        0x5b9cca4f,
	        0x682e6ff3,
	        0x748f82ee,
	        0x78a5636f,
	        0x84c87814,
	        0x8cc70208,
	        0x90befffa,
	        0xa4506ceb,
	        0xbef9a3f7,
	        0xc67178f2,
        ];

        let mut w = [u32::default(); 64];
        let [h0, h1, h2, h3, h4, h5, h6, h7] = self.h;

        while p.len() >= CHUNK {
            for i in 0..16 {
                let j = i * 4;
                w[i] = (p[j] as u32)<<24 | (p[j+1] as u32)<<16 | (p[j+2] as u32)<<8 | (p[j+3] as u32);
            }
            for i in 16..64 {
                let v1 = w[i-2];
                let t1 = v1.rotate_right(17) ^ v1.rotate_right(19) ^ (v1 >> 10);

                let v2 = w[i-15];
                let t2 = v2.rotate_right(7) ^ v2.rotate_right(18) ^ (v2 >> 3);

                w[i] = (t1).wrapping_add(w[i-7]).wrapping_add(t2).wrapping_add(w[i-16]);
            }

            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = [h0, h1, h2, h3, h4, h5, h6, h7];
        }
    }
}
