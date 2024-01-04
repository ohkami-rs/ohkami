use std::borrow::Cow;

const CHUNK:      usize = 64;
const SIZE:       usize = 32/* 256 bits */;
const BLOCK_SIZE: usize = 64;

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
        this.ipad[..secret_key.len()].copy_from_slice(&secret_key);
        this.opad[..secret_key.len()].copy_from_slice(&secret_key);
        for p in &mut this.ipad {
            *p ^= 0x36;
        }
        for p in &mut this.opad {
            *p ^= 0x5c;
        }
        this.inner.write(&this.ipad);

        this
    }

    #[inline] pub fn write(&mut self, p: &[u8]) {
        self.inner.write(p)
    }

    #[inline] pub fn sum(self) -> [u8; SIZE] {
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
    #[inline] pub const fn new() -> Self {
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
            self.nx += n;
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

    #[inline] pub fn sum(mut self) -> [u8; SIZE] {
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
    #[inline] fn reset(&mut self) {
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
        let [mut h0, mut h1, mut h2, mut h3, mut h4, mut h5, mut h6, mut h7] = self.h;

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

            for i in 0..64 {
                let t1 = (h)
                    .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                    .wrapping_add((e & f) ^ (!e & g))
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                    .wrapping_add((a & b) ^ (a & c) ^ (b & c));

                h = g;
                g = f;
                f = e;
                e = (d).wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = (t1).wrapping_add(t2);
            }

            h0 = (h0).wrapping_add(a);
            h1 = (h1).wrapping_add(b);
            h2 = (h2).wrapping_add(c);
            h3 = (h3).wrapping_add(d);
            h4 = (h4).wrapping_add(e);
            h5 = (h5).wrapping_add(f);
            h6 = (h6).wrapping_add(g);
            h7 = (h7).wrapping_add(h);

            p = &p[CHUNK..];
        }

        self.h = [h0, h1, h2, h3, h4, h5, h6, h7];
    }
}




#[cfg(test)] mod test {
    use super::{SHA256, HMAC_SHA256};

    #[test] fn test_sha256() {
        for (expected/* hex literal */, input) in [
            ("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", ""),
        	("ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb", "a"),
        	("fb8e20fc2e4c3f248c60c39bd652f3c1347298bb977b8b4d5903b85055620603", "ab"),
        	("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", "abc"),
        	("88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589", "abcd"),
        	("36bbe50ed96841d10443bcb670d6554f0a34b761be67ec9c4a8ad2c0c44ca42c", "abcde"),
        	("bef57ec7f53a6d40beb640a780a639c83bc29ac8a9816f1fc6c5c6dcd93c4721", "abcdef"),
        	("7d1a54127b222502f5b79b5fb0803061152a44f92b37e23c6527baf665d4da9a", "abcdefg"),
        	("9c56cc51b374c3ba189210d5b6d4bf57790d351c96c47c02190ecf1e430635ab", "abcdefgh"),
        	("19cc02f26df43cc571bc9ed7b0c4d29224a3ec229529221725ef76d021c8326f", "abcdefghi"),
        	("72399361da6a7754fec986dca5b7cbaf1c810a28ded4abaf56b2106d06cb78b0", "abcdefghij"),
        	("a144061c271f152da4d151034508fed1c138b8c976339de229c3bb6d4bbb4fce", "Discard medicine more than two years old."),
        	("6dae5caa713a10ad04b46028bf6dad68837c581616a1589a265a11288d4bb5c4", "He who has a shady past knows that nice guys finish last."),
        	("ae7a702a9509039ddbf29f0765e70d0001177914b86459284dab8b348c2dce3f", "I wouldn't marry him with a ten foot pole."),
        	("6748450b01c568586715291dfa3ee018da07d36bb7ea6f180c1af6270215c64f", "Free! Free!/A trip/to Mars/for 900/empty jars/Burma Shave"),
        	("14b82014ad2b11f661b5ae6a99b75105c2ffac278cd071cd6c05832793635774", "The days of the digital watch are numbered.  -Tom Stoppard"),
        	("7102cfd76e2e324889eece5d6c41921b1e142a4ac5a2692be78803097f6a48d8", "Nepal premier won't resign."),
        	("23b1018cd81db1d67983c5f7417c44da9deb582459e378d7a068552ea649dc9f", "For every action there is an equal and opposite government program."),
        	("8001f190dfb527261c4cfcab70c98e8097a7a1922129bc4096950e57c7999a5a", "His money is twice tainted: 'taint yours and 'taint mine."),
        	("8c87deb65505c3993eb24b7a150c4155e82eee6960cf0c3a8114ff736d69cad5", "There is no reason for any individual to have a computer in their home. -Ken Olsen, 1977"),
        	("bfb0a67a19cdec3646498b2e0f751bddc41bba4b7f30081b0b932aad214d16d7", "It's a tiny change to the code and not completely disgusting. - Bob Manchek"),
        	("7f9a0b9bf56332e19f5a0ec1ad9c1425a153da1c624868fda44561d6b74daf36", "size:  a.out:  bad magic"),
        	("b13f81b8aad9e3666879af19886140904f7f429ef083286195982a7588858cfc", "The major problem is with sendmail.  -Mark Horton"),
        	("b26c38d61519e894480c70c8374ea35aa0ad05b2ae3d6674eec5f52a69305ed4", "Give me a rock, paper and scissors and I will move the world.  CCFestoon"),
        	("049d5e26d4f10222cd841a119e38bd8d2e0d1129728688449575d4ff42b842c1", "If the enemy is within range, then so are you."),
        	("0e116838e3cc1c1a14cd045397e29b4d087aa11b0853fc69ec82e90330d60949", "It's well we cannot hear the screams/That we create in others' dreams."),
        	("4f7d8eb5bcf11de2a56b971021a444aa4eafd6ecd0f307b5109e4e776cd0fe46", "You remind me of a TV show, but that's all right: I watch it anyway."),
        	("61c0cc4c4bd8406d5120b3fb4ebc31ce87667c162f29468b3c779675a85aebce", "C is as portable as Stonehedge!!"),
        	("1fb2eb3688093c4a3f80cd87a5547e2ce940a4f923243a79a2a1e242220693ac", "Even if I could be Shakespeare, I think I should still choose to be Faraday. - A. Huxley"),
        	("395585ce30617b62c80b93e8208ce866d4edc811a177fdb4b82d3911d8696423", "The fugacity of a constituent in a mixture of gases at a given temperature is proportional to its mole fraction.  Lewis-Randall Rule"),
        	("4f9b189a13d030838269dce846b16a1ce9ce81fe63e65de2f636863336a98fe6", "How can you write a big system without C++?  -Paul Glick"),
        ] {
            let sum = std::array::from_fn(|i| i).map(|i|
                [expected.as_bytes()[2*i], expected.as_bytes()[2*i+1]].map(|b| match b {
                    b'0'..=b'9' => b - b'0',
                    b'a'..=b'f' => 10 + b - b'a',
                    _ => unreachable!()
                }).into_iter().fold(0, |byte, b| byte * 2u8.pow(4) + b)
            );

            let mut s = SHA256::new();
            s.write(input.as_bytes());
            assert_eq!(s.sum(), sum);
        }
    }

    #[test] fn test_hmac_sha256() {
        for (key, input, output_hexliteral) in [
            // Tests from RFC 4231
            (
                [
                    0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
                    0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
                    0x0b, 0x0b, 0x0b, 0x0b,
                ].as_slice(),
                "Hi There".as_bytes(),
                "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7".as_bytes(),
            ),
            (
                "Jefe".as_bytes(),
                "what do ya want for nothing?".as_bytes(),
                "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843".as_bytes(),
            ),
            (
                [
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa,
                ].as_slice(),
                [
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
                    0xdd, 0xdd,
                ].as_slice(),
                "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe".as_bytes(),
            ),
            (
                [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
                    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                    0x19,
                ].as_slice(),
                [
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd, 0xcd,
                    0xcd, 0xcd,
                ].as_slice(),
                "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b".as_bytes(),
            ),
            (
                [
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa,
                ].as_slice(),
                "Test Using Larger Than Block-Size Key - Hash Key First".as_bytes(),
                "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54".as_bytes(),
            ),
            (
                [
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
                    0xaa, 0xaa, 0xaa,
                ].as_slice(),
                "This is a test using a larger than block-size key \
                    and a larger than block-size data. The key needs to \
                    be hashed before being used by the HMAC algorithm.".as_bytes(),
                "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2".as_bytes(),
            ),
        
            // Tests from https://csrc.nist.gov/groups/ST/toolkit/examples.html
            // (truncated tag tests are left out)
            (
                [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
                    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
                    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                    0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                ].as_slice(),
                "Sample message for keylen=blocklen".as_bytes(),
                "8bb9a1db9806f20df7f77b82138c7914d174d59e13dc4d0169c9057b133e1d62".as_bytes(),
            ),
            (
                [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                ].as_slice(),
                "Sample message for keylen<blocklen".as_bytes(),
                "a28cf43130ee696a98f14a37678b56bcfcbdd9e5cf69717fecf5480f0ebdf790".as_bytes(),
            ),
            (
                [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
                    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
                    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                    0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
                    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
                    0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
                    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
                    0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
                    0x60, 0x61, 0x62, 0x63,
                ].as_slice(),
                "Sample message for keylen=blocklen".as_bytes(),
                "bdccb6c72ddeadb500ae768386cb38cc41c63dbb0878ddb9c7a38a431b78378d".as_bytes(),
            ),

            // HMAC without key is dumb but should probably not fail.
            (
                &[].as_slice(),
                "message".as_bytes(),
                "eb08c1f56d5ddee07f7bdf80468083da06b64cf4fac64fe3a90883df5feacae4".as_bytes(),
            ),
        ] {
            let expected = std::array::from_fn(|i| i).map(|i|
                [output_hexliteral[2*i], output_hexliteral[2*i+1]].map(|b| match b {
                    b'0'..=b'9' => b - b'0',
                    b'a'..=b'f' => 10 + b - b'a',
                    _ => unreachable!()
                }).into_iter().fold(0, |byte, b| byte * 2u8.pow(4) + b)
            );

            let mut hs = HMAC_SHA256::new(key);
            hs.write(input);
            assert_eq!(hs.sum(), expected);
        }
    }
}
