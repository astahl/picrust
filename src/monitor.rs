use self::format::{Formatting, LeadingZeros};

mod format;
mod parse;

const LINE_LEN: usize = 256;
type Buffer = crate::buffer::Line<u8, LINE_LEN>;

pub struct Monitor<In: Fn() -> u8, Out: Fn(u8)> {
    input: In,
    writer: Writer<Out>,
    line_buffer: Buffer,
    context: CommandContext
}

impl<In: Fn() -> u8, Out: Fn(u8)> Monitor<In, Out> {
    pub fn new(input: In, output: Out) -> Self {
        Self {
            input,
            writer: Writer(output),
            line_buffer: Buffer::new(),
            context: CommandContext {last_address: 0, length: 8}
        }
    }

    pub fn run(&mut self) -> ! {
        self.writer.putc(0x0c);
        self.echo_prompt();
        loop {
            let c = (self.input)().to_ascii_uppercase();
            match c {
                0x7F | 0x08 => {
                    if let Some(_) = self.line_buffer.pop_back() {
                        self.echo_backspace();
                    }
                }
                b'\n' | 0x0D => {
                    self.writer.carriage_return();
                    self.submit();
                    self.line_buffer.clear();
                    self.echo_prompt();
                }
                b'A'..=b'F' | b'0'..=b'9' | b' ' | b'.' | b':' => {
                    if self.line_buffer.push_back(c).is_ok() {
                        self.writer.putc(c);
                    }
                }
                _ => {
                    self.writer.putc(7); // BEL
                    // self.writer.hex(c);
                }
            }
        }
    }

    fn echo_prompt(&self) {
        self.writer.newline();
        let mut formatting = Formatting::default();
        formatting.leading_zeros = LeadingZeros::Skip;
        self.writer.hex_usize(self.context.last_address, Some(formatting));
        self.writer.putc(b'>');
    }

    fn echo_line_buffer(&self) {
        self.writer.puts(self.line_buffer.as_slice());
    }

    fn echo_error(&self, position: usize) {
        self.echo_line_buffer();
        self.writer.newline();
        for _ in 0..position {
            self.writer.putc(b' ');
        }
        self.writer.puts(b"^! Error");
    }

    fn echo_backspace(&self) {
        self.writer.puts(&[0x08, 0x20, 0x08]);
    }

    fn execute(&self, address: usize) -> ! {
        unsafe {
            #[cfg(target_arch = "arm")]
            core::arch::asm!(
                "mov {0}, {1}",
                "blx {0}",
                out(reg) _,
                in(reg) address
            );

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!(
                "blr {0}",
                in(reg) address
            );
        }
        panic!()
    }

    fn submit(&mut self) {

        match Command::parse(self.line_buffer.as_slice()) {
            Ok(mut command) => command.run(&self.writer, &mut self.context),
            Err(err) => {
                match err {
                    CommandParseError::IllegalToken { position } => self.echo_error(position),
                }
            },
        }
    }
}


struct CommandContext {
    pub last_address: usize,
    pub length: usize,
}

enum Command {
    DoNothing,
    ExecuteMemory{start: usize, params: (usize, usize, usize, usize)},
    PrintMemory{start: usize},
    PrintMemoryContinue,
}

pub enum CommandParseError {
    IllegalToken{position: usize}
}

struct Tokenizer<'a> {
    filter_ws: bool,
    position: usize,
    c_str: &'a [u8]
}

impl<'a> Tokenizer<'a> {
    pub fn new(c_str: &'a [u8], filter_ws: bool) -> Self {
        Self { filter_ws, position: 0, c_str }
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
            while self.c_str.get(self.position).is_some_and(u8::is_ascii_whitespace) {
                self.position += 1;
            }
            if !self.filter_ws && self.position != start {
                return Some(Token::new(TokenType::WhiteSpace, start, self.position))
            }
            if self.position >= self.c_str.len() {
                return None
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
                c if c.is_ascii_alphabetic() && !c.is_ascii_hexdigit() => Some(TokenType::SingleLetter(*c)),
                _ => None
            };
            
            if symbol.is_some() {
                self.position = end;
                return symbol.map(|token_type| Token::new(token_type, start, end));
            }

            // try to parse a hex value
            while self.c_str.get(self.position).is_some_and(u8::is_ascii_hexdigit) {
                self.position += 1;
            }
            if self.position != start {
                if let Some(Ok(value)) = self.c_str.get(start..self.position).map(parse::from_hex_be_usize) {
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

enum TokenType {
    Unknown,
    WhiteSpace,
    USize(usize),
    SingleLetter(u8),
    Dot,
    Colon,
    Minus,
    Plus
}

struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, end: usize) -> Self {
        Self { token_type, start, end }
    }
}

impl Command {
    pub fn parse(c_str: &[u8]) -> Result<Command, CommandParseError> {
        let tokenizer = Tokenizer::new(c_str, true);
        for (token_nr, token) in tokenizer.enumerate() {
            match (token_nr, token) {
                (0, Token{token_type: TokenType::USize(value), ..}) => return Ok(Command::PrintMemory { start: value }),
                (_, t) => return Err(CommandParseError::IllegalToken { position: t.start }),
                _ => return Ok(Command::DoNothing)
            }
        }
        return Ok(Command::PrintMemoryContinue);
    }

    pub fn run<Out: Fn(u8)>(&self, out: &Writer<Out>, context: &mut CommandContext) {
        match self {
            Command::DoNothing => {},
            Command::ExecuteMemory { start, params } => {
                unsafe {
                    core::arch::asm!(
                        "mov x0, {1}",
                        "blr {0}",
                        in(reg) start,
                        in(reg) params.0,
                    );
                }
            },
            Command::PrintMemory { start } => {
                self.print_memory(out, *start, context.length);
                context.last_address = *start;
            },
            Command::PrintMemoryContinue => {
                self.print_memory(out, context.last_address, context.length);
                context.last_address += context.length;
            },
        }
    }

    fn print_memory<Out: Fn(u8)>(&self, out: &Writer<Out>, address: usize, length: usize) {
        
        let ptr = address as *const u8;
        let align = core::mem::align_of::<usize>();
        let align_offset = ptr.align_offset(align);
        let from = if align_offset == 0 {
            ptr
        } else {
            let offset = align_offset as isize - core::mem::size_of::<usize>() as isize;
            ptr.wrapping_offset(offset)
        };
        let mut to = ptr.wrapping_add(length);
        let align_offset = to.align_offset(align);
        to = to.wrapping_add(align_offset);
        out.hex_usize(from as usize, Some(Formatting{leading_zeros: LeadingZeros::Space, ..Formatting::default()}));
        out.putc(b':');
        for i in unsafe { from.offset_from(ptr)..to.offset_from(ptr) }{
            if i == 0 || i == length as isize {
                out.putc(b'|');
            } else {
                out.putc(b' ');
            }
            let memvalue = unsafe { core::ptr::read_volatile(ptr.wrapping_offset(i)) };
            out.hex(memvalue, None);
        }
        out.putc(b' ');
        for i in unsafe { from.offset_from(ptr)..to.offset_from(ptr) }{
            let memvalue = unsafe { core::ptr::read_volatile(ptr.wrapping_offset(i)) };
            out.putc(if memvalue.is_ascii_graphic() { memvalue } else { b'.' });
        }
        for i in unsafe { from.offset_from(ptr)..to.offset_from(ptr) }{
            let memvalue = unsafe { core::ptr::read_volatile(ptr.wrapping_offset(i)) };
            out.putc(b' ');
            out.binary(memvalue, None);
        }
        
    }
}

struct Writer<Out: Fn(u8)> (Out);

impl<Out: Fn(u8)> Writer<Out> {
    pub fn putc(&self, char: u8) {
        (self.0)(char);
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

    pub fn binary(&self, value: u8, formatting: Option<Formatting>) {
        let [b7, b6, b5, b4, b3, b2, b1, b0] = format::to_binary(value, &formatting.unwrap_or_default());
        self.putc(b7);
        self.putc(b6);
        self.putc(b5);
        self.putc(b4);
        self.putc(b3);
        self.putc(b2);
        self.putc(b1);
        self.putc(b0);
    }

    pub fn hex(&self, value: u8, formatting: Option<Formatting>) {
        let [upper, lower] = format::to_hex(value, &formatting.unwrap_or_default());
        self.putc(upper);
        self.putc(lower);
    }

    pub fn hex_usize(&self, value: usize, formatting: Option<Formatting>) {
        let formatting = formatting.unwrap_or_default();
        
        match (value, &formatting.leading_zeros) {
            (_, format::LeadingZeros::Keep) => {
                let str = format::to_hex_usize(value, &formatting);
                self.puts_n(&str);
            }
            (0, format::LeadingZeros::Skip) => { self.putc(b'0') }
            (_, format::LeadingZeros::Skip) => {
                for b in format::to_hex_usize(value, &formatting).into_iter().skip_while(|c| *c == b'0') {
                    self.putc(b);
                }
            }
            (0, format::LeadingZeros::Space) => { 
                let str = b"               0";
                self.puts_n(&str)
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
