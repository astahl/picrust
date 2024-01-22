pub enum Case {
    Upper,
    Lower,
}

pub enum LeadingZeros {
    Keep,
    Skip,
    Space
}

pub enum Endianness {
    BE,
    LE,
    Native
}

pub struct Formatting {
    pub prefix: bool,
    pub case: Case,
    pub leading_zeros: LeadingZeros,
    pub endianness: Endianness
}

impl Default for Formatting {
    fn default() -> Formatting {
        Formatting {
            prefix: false,
            case: Case::Lower,
            leading_zeros: LeadingZeros::Keep,
            endianness: Endianness::BE
        }
    }
}

pub const fn to_binary(byte: u8, formatting: &Formatting) -> [u8;8] {
   [((byte >> 7) & 1) | b'0', 
    ((byte >> 6) & 1) | b'0', 
    ((byte >> 5) & 1) | b'0', 
    ((byte >> 4) & 1) | b'0', 
    ((byte >> 3) & 1) | b'0', 
    ((byte >> 2) & 1) | b'0', 
    ((byte >> 1) & 1) | b'0', 
    ((byte >> 0) & 1) | b'0']
}

pub const fn to_hex(byte: u8, formatting: &Formatting) -> [u8;2] {
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

pub const fn to_hex_usize(value: usize, formatting: &Formatting) -> [u8; 16] {
    let mut result: [u8; 16] = [0_u8; 16];
    let bytes = match formatting.endianness {
        Endianness::BE => value.to_be_bytes(),
        Endianness::LE => value.to_le_bytes(),
        Endianness::Native => value.to_ne_bytes(),
    };
    let a = to_hex(bytes[0], formatting);
    let b = to_hex(bytes[1], formatting);
    let c = to_hex(bytes[2], formatting);
    let d = to_hex(bytes[3], formatting);
    let e = to_hex(bytes[4], formatting);
    let f = to_hex(bytes[5], formatting);
    let g = to_hex(bytes[6], formatting);
    let h = to_hex(bytes[7], formatting);
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