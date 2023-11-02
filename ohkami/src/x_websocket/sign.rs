fn sha1(message: &str) -> [u8; 160] {
    //let message_bits =  message.as_bytes();
//
    //u32::from_be_bytes(bytes)
//
    //let ml = message.len() as u64;
//
    //let mut h0: u32 = 0x67452301;
    //let mut h1: u32 = 0xEFCDAB89;
    //let mut h2: u32 = 0x98BADCFE;
    //let mut h3: u32 = 0x10325476;
    //let mut h4: u32 = 0xC3D2E1F0;
//
    ////let message_u16 = message as u64;
//
    todo!()
}

const CHANK: usize = 64;
struct Digest {
    h:   [u32; 5],
    x:   [u8; CHANK],
    nx:  usize,
    len: u64,
}

const K0: u32 = 0x5A827999;
const K1: u32 = 0x6ED9EBA1;
const K2: u32 = 0x8F1BBCDC;
const K3: u32 = 0xCA62C1D6;

impl Digest {
    fn reset() -> Self {
        Self {
            h:   [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            x:   [0; CHANK],
            nx:  0,
            len: 0,
        }
    }

    fn write(&mut self, mut data: &[u8]) {
        self.len += data.len() as u64;
        if self.nx > 0 {
            let n = (CHANK - self.nx).min(data.len());
            self.x[self.nx..(self.nx + n)].copy_from_slice(data);
            self.nx += n;
            if self.nx == CHANK {
                self.block_x();
                self.nx = 0;
            }
            data = &data[n..]
        }
    }

    fn block(&mut self, mut data: &[u8]) {

    }

    fn block_x(&mut self) {
        let mut w = [0u32; 16];

        let (h0, h1, h2, h3, h4) = (self.h[0], self.h[1], self.h[2], self.h[3], self.h[4]);
        while self.x.len() >= CHANK {
            for i in 0..16 {
                let j = i * 4;
                w[i] = (self.x[j] as u32) << 24 | (self.x[j+1] as u32) << 16 | (self.x[j+2] as u32) << 8 | (self.x[j+3] as u32);
            }

            let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);

            for i in 0..16 {
                let f = (b & c) | ((!b) & d);
                let t = a.rotate_left(5) + f + e + w[i & 0xf] + K0;
                (a, b, c, d, e) = (t, a, b.rotate_left(30), c, d)
            }
            for i in 16..20 {
                tmp := 
            }
        }
    }
}
