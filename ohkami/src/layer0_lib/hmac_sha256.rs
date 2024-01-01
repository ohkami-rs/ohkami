const CHUNK: usize = 64;
const SIZE:  usize = 32/* 256 bits */;

pub struct HMACSha256 {
    h:   [u32; 8],
    x:   [u8; CHUNK],
    nx:  usize,
    len: usize,
}

impl HMACSha256 {
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
        todo!()
    }

    pub fn sum(self) -> [u8; SIZE] {
        todo!()
    }
}
