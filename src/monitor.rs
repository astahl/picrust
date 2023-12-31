const LINE_LEN: usize = 256;

const fn to_hex_digit(nybble: u8) -> u8 {
    let char = nybble + b'0';
    if char > b'9' {
        char + 7
    } else {
        char
    }
}

const fn nybble_from_hex_digit(char: u8) -> u8 {
    let value = char - b'0';
    if value > 9 {
        value - 7
    } else {
        value
    }
}

pub struct Monitor<In: Fn() -> u8, Out: Fn(u8)> {
    input: In,
    output: Out,
    line_buffer: [u8; LINE_LEN],
    address: usize,
    line_cursor: usize,
}

impl<In: Fn() -> u8, Out: Fn(u8)> Monitor<In, Out> {
    pub fn new(input: In, output: Out) -> Self {
        Self {
            input,
            output,
            line_buffer: [0; LINE_LEN],
            address: 0,
            line_cursor: 0,
        }
    }

    pub fn run(&mut self) {
        self.echo_newline();
        loop {
            let c = (self.input)().to_ascii_uppercase();
            match c {
                0x7F | 0x08 => {
                    if self.line_cursor != 0 {
                        self.line_buffer[self.line_cursor] = 0;
                        self.line_cursor -= 1;
                        self.echo_backspace();
                    }
                }
                b'\n' | 0x0D => {
                    self.submit();
                }
                b'A'..=b'Z' | b'0'..=b'9' | b' ' | b'.' | b':' => {
                    self.echo(c);
                    self.line_buffer[self.line_cursor] = c;
                    if self.line_cursor < LINE_LEN - 1 {
                        self.line_cursor += 1;
                    }
                }
                _ => {
                    self.echo(7);
                    self.echo_hex(c);
                }
            }
        }
    }

    fn echo(&self, ascii: u8) {
        (self.output)(ascii);
    }

    fn echo_hex(&self, value: u8) {
        self.echo(to_hex_digit(value >> 4));
        self.echo(to_hex_digit(value & 0xF));
    }

    fn echo_newline(&self) {
        self.echo(b'\n');
    }

    fn echo_line_buffer(&self) {
        for c in self.line_buffer.iter().take_while(|c| **c != 0) {
            self.echo(*c);
        }
    }

    fn echo_error(&self, position: usize) {
        self.echo_line_buffer();
        self.echo_newline();
        for _ in 0..position {
            self.echo(b' ');
        }
        self.echo(b'^');
        self.echo(b'!');
    }

    fn echo_backspace(&self) {
        self.echo(0x08);
        self.echo(0x20);
        self.echo(0x08);
    }

    fn echo_memory(&self, address: usize, len: usize) {
        for b in address.to_be_bytes() {
            self.echo_hex(b);
        }

        self.echo(b':');
        self.echo(b' ');

        for i in 0..len {
            let ptr = address as *const u8;
            let memvalue = unsafe { core::ptr::read_volatile(ptr.add(i)) };
            self.echo_hex(memvalue);
            self.echo(b' ');
        }
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
        self.echo_newline();
        // parse the line in the buffer
        self.line_cursor = 0;
        let mut current: usize = 0;
        let mut mode = 'P';
        for (pos, c) in self.line_buffer.iter().enumerate() {
            match *c {
                b'R' => {
                    mode = 'R';
                }
                b' ' => {
                    current = 0;
                }
                0 => {
                    break;
                }
                b'0'..=b'9' | b'A'..=b'F' => {
                    let n = nybble_from_hex_digit(*c) & 0xF;
                    current <<= 4;
                    current |= n as usize;
                }
                _ => {
                    self.echo_error(pos);
                    self.echo_newline();
                    return;
                }
            }
        }
        if current > 0 {
            self.address = current;
        }
        match mode {
            'R' => self.execute(current),
            _ => self.echo_memory(current, 8),
        };

        self.echo_newline();
    }
}
