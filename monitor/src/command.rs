use crate::token;
use crate::writer;
use mystd::format;

pub struct CommandContext {
    pub last_address: usize,
    pub length: usize,
    pub cursor_type: CursorType,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self { last_address: 0, length: 8, cursor_type: CursorType::U64 }
    }
}

pub enum Command {
    DoNothing,
    ExecuteMemory {
        start: usize,
        params: (usize, usize, usize, usize),
    },
    PrintMemory {
        start: usize,
    },
    PrintMemoryContinue,
}

pub enum CommandParseError {
    IllegalToken { position: usize },
}


impl Command {
    pub fn parse(c_str: &[u8]) -> Result<Command, CommandParseError> {
        let tokenizer = token::Tokenizer::new(c_str, true);
        for (token_nr, token) in tokenizer.enumerate() {
            match (token_nr, token) {
                (
                    0,
                    token::Token {
                        token_type: token::TokenType::USize(value),
                        ..
                    },
                ) => return Ok(Command::PrintMemory { start: value }),
                (_, t) => return Err(CommandParseError::IllegalToken { position: t.start }),
                _ => return Ok(Command::DoNothing),
            }
        }
        return Ok(Command::PrintMemoryContinue);
    }

    pub fn run<Out: Fn(u8)>(&self, out: &writer::Writer<Out>, context: &mut CommandContext) {
        match self {
            Command::DoNothing => {}
            Command::ExecuteMemory { start, params } => unsafe {
                core::arch::asm!(
                    "mov x0, {1}",
                    "blr {0}",
                    in(reg) start,
                    in(reg) params.0,
                );
            },
            Command::PrintMemory { start } => {
                context.last_address =
                    self.print_memory(out, *start, context.length, context.cursor_type);
            }
            Command::PrintMemoryContinue => {
                context.last_address = self.print_memory(
                    out,
                    context.last_address,
                    context.length,
                    context.cursor_type,
                );
            }
        }
    }

    fn print_memory<Out: Fn(u8)>(
        &self,
        out: &writer::Writer<Out>,
        address: usize,
        length: usize,
        cursor: CursorType,
    ) -> usize {
        // calculate start and end of the column we print
        let cur_start = address;
        let cur_end = address + cursor.byte_len();
        let column_align = cursor.align_of().max(length);
        let mut start = address & !(column_align - 1);
        let mut end = start + length;
        loop {
            // print the address of the column first
            out.hex_usize(
                start,
                Some(format::Formatting {
                    leading_zeros: format::LeadingZeros::Space,
                    ..format::Formatting::default()
                }),
            );
            out.putc(b':');

            for addr in start..end {
                if addr == cur_start {
                    out.putc(b'(');
                } else if addr == cur_end {
                    out.putc(b')');
                } else {
                    out.putc(b' ');
                }
                let memvalue = unsafe { (addr as *const u8).read_volatile() };
                out.hex(memvalue, None);
            }
            if cur_end == end {
                out.putc(b')');
            } else {
                out.putc(b' ');
            }

            for addr in start..end {
                let memvalue = unsafe { (addr as *const u8).read_volatile() };
                out.putc(if memvalue.is_ascii_graphic() {
                    memvalue
                } else {
                    b'.'
                });
            }

            for addr in cur_start.max(start)..cur_end.min(end) {
                let memvalue = unsafe { (addr as *const u8).read_volatile() };
                out.putc(b' ');
                out.binary(memvalue, None);
            }

            if cur_end <= end {
                break;
            } else {
                end += length;
                start += length;
                out.newline();
            }
        }
        cur_end
    }
}


#[derive(Clone, Copy)]
pub enum CursorType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl CursorType {
    pub const fn byte_len(&self) -> usize {
        match self {
            CursorType::U8 => core::mem::size_of::<u8>(),
            CursorType::U16 => core::mem::size_of::<u16>(),
            CursorType::U32 => core::mem::size_of::<u32>(),
            CursorType::U64 => core::mem::size_of::<u64>(),
            CursorType::U128 => core::mem::size_of::<u128>(),
        }
    }

    pub const fn align_of(&self) -> usize {
        match self {
            CursorType::U8 => core::mem::align_of::<u8>(),
            CursorType::U16 => core::mem::align_of::<u16>(),
            CursorType::U32 => core::mem::align_of::<u32>(),
            CursorType::U64 => core::mem::align_of::<u64>(),
            CursorType::U128 => core::mem::align_of::<u128>(),
        }
    }

    pub const fn wider(&self) -> Self {
        match self {
            CursorType::U8 => CursorType::U16,
            CursorType::U16 => CursorType::U32,
            CursorType::U32 => CursorType::U64,
            CursorType::U64 => CursorType::U128,
            CursorType::U128 => CursorType::U128,
        }
    }

    pub const fn slimmer(&self) -> Self {
        match self {
            CursorType::U8 => CursorType::U8,
            CursorType::U16 => CursorType::U8,
            CursorType::U32 => CursorType::U16,
            CursorType::U64 => CursorType::U32,
            CursorType::U128 => CursorType::U64,
        }
    }
}
