pub enum Case {
    Upper,
    Lower,
}

pub enum LeadingZeros {
    Keep,
    Skip,
    Space,
}

pub enum Endianness {
    BE,
    LE,
    Native,
}

pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

pub struct Formatting {
    pub prefix: bool,
    pub case: Case,
    pub leading_zeros: LeadingZeros,
    pub endianness: Endianness,
    pub bit_order: BitOrder,
}

impl Default for Formatting {
    fn default() -> Formatting {
        Formatting {
            prefix: false,
            case: Case::Lower,
            leading_zeros: LeadingZeros::Keep,
            endianness: Endianness::BE,
            bit_order: BitOrder::MsbFirst,
        }
    }
}

pub const fn to_binary(byte: u8, formatting: &Formatting) -> [u8; 8] {
    match formatting.bit_order {
        BitOrder::MsbFirst => [
            ((byte >> 7) & 1) | b'0',
            ((byte >> 6) & 1) | b'0',
            ((byte >> 5) & 1) | b'0',
            ((byte >> 4) & 1) | b'0',
            ((byte >> 3) & 1) | b'0',
            ((byte >> 2) & 1) | b'0',
            ((byte >> 1) & 1) | b'0',
            ((byte >> 0) & 1) | b'0',
        ],
        BitOrder::LsbFirst => [
            ((byte >> 0) & 1) | b'0',
            ((byte >> 1) & 1) | b'0',
            ((byte >> 2) & 1) | b'0',
            ((byte >> 3) & 1) | b'0',
            ((byte >> 4) & 1) | b'0',
            ((byte >> 5) & 1) | b'0',
            ((byte >> 6) & 1) | b'0',
            ((byte >> 7) & 1) | b'0',
        ],
    }
}

pub const fn hex_len<T>() -> usize {
    core::mem::size_of::<T>() * 2
}

pub const fn hex_len_val<T>(_: &T) -> usize {
    hex_len::<T>()
}

pub const fn to_hex_u8(byte: u8, formatting: &Formatting) -> [u8; 2] {
    const SYMBOLS: &[u8; 16] = b"0123456789abcdef";
    const SYMBOLS_UPPER: &[u8; 16] = b"0123456789ABCDEF";
    let symbols = match formatting.case {
        Case::Upper => SYMBOLS_UPPER,
        Case::Lower => SYMBOLS,
    };
    let upper = (byte >> 4) & 0xF;
    let lower = byte & 0xF;
    [symbols[upper as usize], symbols[lower as usize]]
}

pub const fn to_hex_u16(value: u16, formatting: &Formatting) -> [u8; 4] {
    let mut result: [u8; 4] = [0_u8; 4];
    let bytes = match formatting.endianness {
        Endianness::BE => value.to_be_bytes(),
        Endianness::LE => value.to_le_bytes(),
        Endianness::Native => value.to_ne_bytes(),
    };
    let a = to_hex_u8(bytes[0], formatting);
    let b = to_hex_u8(bytes[1], formatting);
    result[0] = a[0];
    result[1] = a[1];
    result[2] = b[0];
    result[3] = b[1];
    result
}

pub const fn to_hex_u32(value: u32, formatting: &Formatting) -> [u8; 8] {
    let mut result: [u8; 8] = [0_u8; 8];
    let bytes = match formatting.endianness {
        Endianness::BE => value.to_be_bytes(),
        Endianness::LE => value.to_le_bytes(),
        Endianness::Native => value.to_ne_bytes(),
    };
    let a = to_hex_u8(bytes[0], formatting);
    let b = to_hex_u8(bytes[1], formatting);
    let c = to_hex_u8(bytes[2], formatting);
    let d = to_hex_u8(bytes[3], formatting);
    result[0] = a[0];
    result[1] = a[1];
    result[2] = b[0];
    result[3] = b[1];
    result[4] = c[0];
    result[5] = c[1];
    result[6] = d[0];
    result[7] = d[1];
    result
}

pub const fn to_hex_u64(value: u64, formatting: &Formatting) -> [u8; 16] {
    let mut result: [u8; 16] = [0_u8; 16];
    let bytes = match formatting.endianness {
        Endianness::BE => value.to_be_bytes(),
        Endianness::LE => value.to_le_bytes(),
        Endianness::Native => value.to_ne_bytes(),
    };
    let a = to_hex_u8(bytes[0], formatting);
    let b = to_hex_u8(bytes[1], formatting);
    let c = to_hex_u8(bytes[2], formatting);
    let d = to_hex_u8(bytes[3], formatting);
    let e = to_hex_u8(bytes[4], formatting);
    let f = to_hex_u8(bytes[5], formatting);
    let g = to_hex_u8(bytes[6], formatting);
    let h = to_hex_u8(bytes[7], formatting);
    result[0] = a[0];
    result[1] = a[1];
    result[2] = b[0];
    result[3] = b[1];
    result[4] = c[0];
    result[5] = c[1];
    result[6] = d[0];
    result[7] = d[1];
    result[8] = e[0];
    result[9] = e[1];
    result[10] = f[0];
    result[11] = f[1];
    result[12] = g[0];
    result[13] = g[1];
    result[14] = h[0];
    result[15] = h[1];
    result
}

#[cfg(target_pointer_width = "64")]
pub const fn to_hex_usize(value: usize, formatting: &Formatting) -> [u8; 16] {
    to_hex_u64(value as u64, formatting)
}

#[cfg(target_pointer_width = "32")]
pub const fn to_hex_usize(value: usize, formatting: &Formatting) -> [u8; 8] {
    to_hex_u32(value as u32, formatting)
}

pub const fn to_hex_u128(value: u128, formatting: &Formatting) -> [u8; 32] {
    let mut result: [u8; 32] = [0_u8; 32];
    let bytes = match formatting.endianness {
        Endianness::BE => value.to_be_bytes(),
        Endianness::LE => value.to_le_bytes(),
        Endianness::Native => value.to_ne_bytes(),
    };
    let a = to_hex_u8(bytes[0], formatting);
    let b = to_hex_u8(bytes[1], formatting);
    let c = to_hex_u8(bytes[2], formatting);
    let d = to_hex_u8(bytes[3], formatting);
    let e = to_hex_u8(bytes[4], formatting);
    let f = to_hex_u8(bytes[5], formatting);
    let g = to_hex_u8(bytes[6], formatting);
    let h = to_hex_u8(bytes[7], formatting);
    let i = to_hex_u8(bytes[8], formatting);
    let j = to_hex_u8(bytes[9], formatting);
    let k = to_hex_u8(bytes[10], formatting);
    let l = to_hex_u8(bytes[11], formatting);
    let m = to_hex_u8(bytes[12], formatting);
    let n = to_hex_u8(bytes[13], formatting);
    let o = to_hex_u8(bytes[14], formatting);
    let p = to_hex_u8(bytes[15], formatting);
    result[0] = a[0];
    result[1] = a[1];
    result[2] = b[0];
    result[3] = b[1];
    result[4] = c[0];
    result[5] = c[1];
    result[6] = d[0];
    result[7] = d[1];
    result[8] = e[0];
    result[9] = e[1];
    result[10] = f[0];
    result[11] = f[1];
    result[12] = g[0];
    result[13] = g[1];
    result[14] = h[0];
    result[15] = h[1];
    result[16] = i[0];
    result[17] = i[1];
    result[18] = j[0];
    result[19] = j[1];
    result[20] = k[0];
    result[21] = k[1];
    result[22] = l[0];
    result[23] = l[1];
    result[24] = m[0];
    result[25] = m[1];
    result[26] = n[0];
    result[27] = n[1];
    result[28] = o[0];
    result[29] = o[1];
    result[30] = p[0];
    result[31] = p[1];
    result
}

const fn to_decimal_u8(mut num: u8) -> [u8; 3] {
    let mut result = [b'0'; 3];
    const POWERS: [u8; 2] = [100, 10];
    let mut i = 0;
    let mut n = 1;
    while i < result.len() - 1 {
        let mut modifier = 0;
        while n >= 0 {
            let reducer = POWERS[i] << n;
            if num >= reducer {
                modifier |= 1 << n;
                num -= reducer;
            }
            n -= 1;
        }
        result[i] |= modifier;
        i += 1;
        n = 3;
    }
    result[i] += num as u8;
    result
}

const fn to_decimal_u8_skip_first_zero_unchecked(mut num: u8) -> [u8; 2] {
    let mut result = [b'0'; 2];
    let mut n = 3;
    let mut modifier = 0;
    while n >= 0 {
        let reducer = 10 << n;
        if num >= reducer {
            modifier |= 1 << n;
            num -= reducer;
        }
        n -= 1;
    }
    result[0] |= modifier;
    result[1] += num as u8;
    result
}

const fn to_decimal_u16(mut num: u16) -> [u8; 5] {
    let mut result = [b'0'; 5];
    const POWERS: [u16; 4] = [10000, 1000, 100, 10];
    let mut i = 0;
    let mut n = 2; // highest most significant digit is 6, which is larger than 2^2
    while i < result.len() - 1 {
        let mut modifier = 0;
        while n >= 0 {
            let reducer = POWERS[i] << n;
            if num >= reducer {
                modifier |= 1 << n;
                num -= reducer;
            }
            n -= 1;
        }
        result[i] |= modifier;
        i += 1;
        n = 3; // reset to 2^3 for all following digits
    }
    result[i] += num as u8;
    result
}

const fn to_decimal_u32(mut num: u32) -> [u8; 10] {
    let mut result = [b'0'; 10];
    const POWERS: [u32; 9] = [
        1000000000, 100000000, 10000000, 1000000, 100000, 10000, 1000, 100, 10,
    ];
    let mut i = 0;
    let mut n = 2; // highest most significant digit is 4 >= 2^2
    while i < result.len() - 1 {
        let mut modifier = 0;
        while n >= 0 {
            let reducer = POWERS[i] << n;
            if num >= reducer {
                modifier |= 1 << n;
                num -= reducer;
            }
            n -= 1;
        }
        result[i] |= modifier;
        i += 1;
        n = 3; // reset to 2^3 for all following digits
    }
    result[i] += num as u8;
    result
}

const fn to_decimal_u32_skip_first_zero_unchecked(mut num: u32) -> [u8; 9] {
    let mut result = [b'0'; 9];
    const POWERS: [u32; 8] = [100000000, 10000000, 1000000, 100000, 10000, 1000, 100, 10];
    let mut i = 0;
    while i < result.len() - 1 {
        let mut n = 3; // reset to 2^3 for all following digits
        let mut modifier = 0;
        while n >= 0 {
            let reducer = POWERS[i] << n;
            if num >= reducer {
                modifier |= 1 << n;
                num -= reducer;
            }
            n -= 1;
        }
        result[i] |= modifier;
        i += 1;
    }
    result[i] += num as u8;
    result
}

const fn to_decimal_u64(num: u64) -> [u8; 20] {
    let low9 = (num % 1000000000) as u32;
    let upper = (num / 1000000000);
    let mid9 = (upper % 1000000000) as u32;
    let top2 = (upper / 1000000000) as u8;

    unsafe {
        core::mem::transmute((
            to_decimal_u8_skip_first_zero_unchecked(top2),
            to_decimal_u32_skip_first_zero_unchecked(mid9),
            to_decimal_u32_skip_first_zero_unchecked(low9),
        ))
    }
}

#[cfg(target_pointer_width = "64")]
pub const fn to_decimal_usize(value: usize) -> [u8; 20] {
    to_decimal_u64(value as u64)
}

#[cfg(target_pointer_width = "32")]
pub const fn to_decimal_usize(value: usize) -> [u8; 10] {
    to_decimal_u32(value as u32)
}
