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
} impl HMAC_SHA256 {
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
} impl SHA256 {
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

    pub fn write(&mut self, p: &[u8]) {
        let nn = p.len();

        self.len += nn;

        if self.nx > 0 {
            let n = usize::min(self.x.len() - self.nx, p.len());
            self.x[self.nx..(self.nx + n)].copy_from_slice(p);
            if self.nx == CHUNK {
                
            }
        }

        todo!()
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
} impl SHA256 {
    fn reset(&mut self) {
        todo!()
    }

    fn marshal_binary(&mut self) -> [u8; MARSHALIZED_SIZE] {
        todo!()
    }
}
