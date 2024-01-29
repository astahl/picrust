use mystd::format;

pub struct Writer<Out: Fn(u8)>(Out);

impl<Out: Fn(u8)> Writer<Out> {

    pub fn new(output: Out) -> Self {
        Self(output)
    }

    pub fn putc(&self, char: u8) {
        (self.0)(char);
    }

    pub fn putc_repeat(&self, char: u8, mut count: usize) {
        while count > 0 {
            self.putc(char);
            count -= 1;
        }
    }

    pub fn puts_n<const N: usize>(&self, str: &[u8; N]) {
        for i in 0..N {
            self.putc(str[i]);
        }
    }

    pub fn puts(&self, str: &[u8]) {
        for c in str {
            self.putc(*c);
        }
    }

    pub fn put_iter<I: Iterator<Item = u8>>(&self, iter: I) {
        iter.for_each(|c| self.putc(c));
    }

    pub fn binary(&self, value: u8, formatting: Option<format::Formatting>) {
        let [b7, b6, b5, b4, b3, b2, b1, b0] =
            format::to_binary(value, &formatting.unwrap_or_default());
        self.putc(b7);
        self.putc(b6);
        self.putc(b5);
        self.putc(b4);
        self.putc(b3);
        self.putc(b2);
        self.putc(b1);
        self.putc(b0);
    }

    pub fn decimal_usize(&self, value: usize, formatting: Option<format::Formatting>) {
        let formatting = formatting.unwrap_or_default();

        match (value, &formatting.leading_zeros) {
            (_, format::LeadingZeros::Keep) => {
                let str = format::to_decimal_usize(value);
                self.puts_n(&str);
            }
            (0, format::LeadingZeros::Skip) => self.putc(b'0'),
            (_, format::LeadingZeros::Skip) => {
                for b in format::to_decimal_usize(value)
                    .into_iter()
                    .skip_while(|c| *c == b'0')
                {
                    self.putc(b);
                }
            }
            (0, format::LeadingZeros::Space) => {
                self.putc_repeat(b' ', 20);
                self.putc(b'0');
            }
            (_, format::LeadingZeros::Space) => {
                let mut str = format::to_decimal_usize(value);
                for b in str.iter_mut() {
                    if *b == b'0' {
                        *b = b' ';
                    } else {
                        break;
                    }
                }
                self.puts_n(&str);
            }
        }
    }

    pub fn hex(&self, value: u8, formatting: Option<format::Formatting>) {
        let [upper, lower] = format::to_hex_u8(value, &formatting.unwrap_or_default());
        self.putc(upper);
        self.putc(lower);
    }

    pub fn hex_usize(&self, value: usize, formatting: Option<format::Formatting>) {
        let formatting = formatting.unwrap_or_default();

        match (value, &formatting.leading_zeros) {
            (_, format::LeadingZeros::Keep) => {
                let str = format::to_hex_usize(value, &formatting);
                self.puts_n(&str);
            }
            (0, format::LeadingZeros::Skip) => self.putc(b'0'),
            (_, format::LeadingZeros::Skip) => {
                for b in format::to_hex_usize(value, &formatting)
                    .into_iter()
                    .skip_while(|c| *c == b'0')
                {
                    self.putc(b);
                }
            }
            (0, format::LeadingZeros::Space) => {
                self.putc_repeat(b' ', format::hex_len_val(&value) - 1);
                self.putc(b'0');
            }
            (_, format::LeadingZeros::Space) => {
                let mut str = format::to_hex_usize(value, &formatting);
                for b in str.iter_mut() {
                    if *b == b'0' {
                        *b = b' ';
                    } else {
                        break;
                    }
                }
                self.puts_n(&str);
            }
        }
    }

    pub fn newline(&self) {
        self.putc(b'\n');
    }

    pub fn carriage_return(&self) {
        self.putc(b'\r');
    }
}
