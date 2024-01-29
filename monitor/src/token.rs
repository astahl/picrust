use mystd::parse;

pub struct Tokenizer<'a> {
    filter_ws: bool,
    position: usize,
    c_str: &'a [u8],
}

impl<'a> Tokenizer<'a> {
    pub fn new(c_str: &'a [u8], filter_ws: bool) -> Self {
        Self {
            filter_ws,
            position: 0,
            c_str,
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == self.c_str.len() {
            None
        } else {
            let start = self.position;
            // handle whitespace
            while self
                .c_str
                .get(self.position)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.position += 1;
            }
            if !self.filter_ws && self.position != start {
                return Some(Token::new(TokenType::WhiteSpace, start, self.position));
            }
            if self.position >= self.c_str.len() {
                return None;
            }

            let start = self.position;

            // try to parse symbols, we checked that we're not beyond the buffer above.
            let c = unsafe { self.c_str.get_unchecked(self.position) };
            let end = self.position + 1;
            let symbol = match c {
                b':' => Some(TokenType::Colon),
                b'.' => Some(TokenType::Dot),
                b'+' => Some(TokenType::Plus),
                b'-' => Some(TokenType::Minus),
                c if c.is_ascii_alphabetic() && !c.is_ascii_hexdigit() => {
                    Some(TokenType::SingleLetter(*c))
                }
                _ => None,
            };

            if symbol.is_some() {
                self.position = end;
                return symbol.map(|token_type| Token::new(token_type, start, end));
            }

            // try to parse a hex value
            while self
                .c_str
                .get(self.position)
                .is_some_and(u8::is_ascii_hexdigit)
            {
                self.position += 1;
            }
            if self.position != start {
                if let Some(Ok(value)) = self
                    .c_str
                    .get(start..self.position)
                    .map(parse::from_hex_be_usize)
                {
                    return Some(Token::new(TokenType::USize(value), start, self.position));
                } else {
                    return Some(Token::new(TokenType::Unknown, start, self.position));
                }
            }

            self.position += 1;
            Some(Token::new(TokenType::Unknown, start, self.position))
        }
    }
}

pub enum TokenType {
    Unknown,
    WhiteSpace,
    USize(usize),
    SingleLetter(u8),
    Dot,
    Colon,
    Minus,
    Plus,
}

pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, end: usize) -> Self {
        Self {
            token_type,
            start,
            end,
        }
    }
}