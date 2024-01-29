#![no_std]

use mystd::format;
mod writer;
mod token;
mod command;
use writer::Writer;

const LINE_LEN: usize = 256;
type Buffer = mystd::buffer::Line<u8, LINE_LEN>;

pub struct Monitor<In: Fn() -> u8, Out: Fn(u8)> {
    input: In,
    writer: Writer<Out>,
    line_buffer: Buffer,
    context: command::CommandContext,
}

impl<In: Fn() -> u8, Out: Fn(u8)> Monitor<In, Out> {
    pub fn new(input: In, output: Out) -> Self {
        Self {
            input,
            writer: Writer::new(output),
            line_buffer: Buffer::new(),
            context: command::CommandContext::default(),
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
                b'+' => {
                    self.context.cursor_type = self.context.cursor_type.wider();
                }
                b'-' => {
                    self.context.cursor_type = self.context.cursor_type.slimmer();
                }
                c if c.is_ascii_hexdigit() => {
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
        let mut formatting = format::Formatting::default();
        formatting.leading_zeros = format::LeadingZeros::Skip;
        self.writer
            .hex_usize(self.context.last_address, Some(formatting));
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
        match command::Command::parse(self.line_buffer.as_slice()) {
            Ok(command) => command.run(&self.writer, &mut self.context),
            Err(err) => match err {
                command::CommandParseError::IllegalToken { position } => self.echo_error(position),
            },
        }
    }
}

