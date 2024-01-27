pub struct Bcd<const LEN: usize>([u8; LEN]);

impl<const LEN: usize> Bcd<LEN> {
    pub const fn from_u8(mut num: u8) -> Bcd<2> {
        let mut result = [0; 2];
        const POWERS: [u8; 2] = [100, 10];
        let mut i = 0;
        let mut n = 1;
        while i < 3 {
            let mut modifier = 0;
            while n >= 0 {
                let reducer = POWERS[i] << n;
                if num >= reducer {
                    modifier |= 1 << n;
                    num -= reducer;
                }
                n -= 1;
            }
            result[i >> 1] |= modifier << ((1 - (i & 1)) << 2);
            i += 1;
            n = 4;
        }
        result[i] |= num as u8;
        Bcd(result)
    }
}
