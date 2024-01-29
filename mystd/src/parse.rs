pub enum ParseError {
    OutOfRange,
    InvalidCharacter,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub const fn from_hex(digit: u8) -> ParseResult<u8> {
    let mut value = digit.wrapping_sub(b'0');
    if value < 0xA {
        Ok(value)
    } else {
        value -= 7;
        if value < 0x10 {
            Ok(value)
        } else {
            Err(ParseError::OutOfRange)
        }
    }
}

pub const fn from_hex_u8(str: &[u8]) -> ParseResult<u8> {
    let mut result = 0_u8;

    let i = 0;
    while i < str.len() {
        if result < 0x10 {
            if let Ok(val) = from_hex(str[i]) {
                result <<= 4;
                result |= val;
            } else {
                return Err(ParseError::InvalidCharacter);
            }
        } else {
            return Err(ParseError::OutOfRange);
        }
    }
    return Ok(result);
}

pub const fn from_hex_be_usize(str: &[u8]) -> ParseResult<usize> {
    let mut result = 0_usize;
    const MAX_BEFORE_SHIFT: usize = (usize::MAX >> 4) + 1;

    let mut i = 0;
    while i < str.len() {
        if result < MAX_BEFORE_SHIFT {
            if let Ok(val) = from_hex(str[i]) {
                result <<= 4;
                result |= val as usize;
                i += 1;
            } else {
                return Err(ParseError::InvalidCharacter);
            }
        } else {
            return Err(ParseError::OutOfRange);
        }
    }
    return Ok(result);
}
