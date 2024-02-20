use core::fmt::{Display, Write};

use crate::peripherals::mailbox;

pub struct ByteSize(pub usize);

impl Display for ByteSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const MASK: usize = 0x3ff;
        const NAMES: [&'static str; 7] = [
            "Exbibyte", "Pebibyte", "Tebibyte", "Gibibyte", "Mebibyte", "Kibibyte", "Byte",
        ];
        let separator = "+";
        let mut needs_separator = false;
        f.write_char('(');
        for i in 0..7 {
            let val = (self.0 >> ((6 - i) * 10)) & MASK;
            if val != 0 {
                write!(
                    f,
                    "{}{} {}",
                    if needs_separator { separator } else { "" },
                    val,
                    NAMES[i]
                )?;
                needs_separator = true;
            }
            if val > 1 {
                f.write_char('s')?;
            }
        }
        f.write_char(')');
        Ok(())
    }
}

pub struct Memory {
    pub base_address: usize,
    pub size: ByteSize,
}

impl Display for Memory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Memory[{} starting at 0x{:x}]",
            self.size, self.base_address
        )
    }
}

#[derive(Debug)]
pub enum Type {
    Pi1A,
    Pi1B,
    Pi1APlus,
    Pi1BPlus,
    Pi2B,
    Alpha,
    CM1,
    Unused7,
    Pi3B,
    Zero,
    CM3,
    UnusedB,
    ZeroW,
    Pi3BPlus,
    Pi3APlus,
    InternalF,
    CM3Plus,
    Pi4B,
    Zero2W,
    Pi400,
    CM4,
    CM4S,
    Internal16,
    Pi5,
}

impl Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Type::Pi1A => write!(f, "A"),
            Type::Pi1B => write!(f, "B"),
            Type::Pi1APlus => write!(f, "A+"),
            Type::Pi1BPlus => write!(f, "B+"),
            Type::Pi2B => write!(f, "2B"),
            Type::Alpha => write!(f, "Alpha"),
            Type::CM1 => write!(f, "CM1"),
            Type::Pi3B => write!(f, "3B"),
            Type::Zero => write!(f, "Zero"),
            Type::CM3 => write!(f, "CM3"),
            Type::ZeroW => write!(f, "Zero W"),
            Type::Pi3BPlus => write!(f, "3B+"),
            Type::Pi3APlus => write!(f, "3A+"),
            Type::CM3Plus => write!(f, "CM3+"),
            Type::Pi4B => write!(f, "4B"),
            Type::Zero2W => write!(f, "Zero 2 W"),
            Type::Pi400 => write!(f, "400"),
            Type::CM4 => write!(f, "CM4"),
            Type::CM4S => write!(f, "CM4S"),
            Type::Pi5 => write!(f, "5"),
            _ => write!(f, "Unknown Model"),
        }
    }
}

#[derive(Debug)]
pub enum Manufacturer {
    SonyUK,
    Egoman,
    Embest,
    SonyJapan,
    Embest2,
    Stadium,
    Qisda,
}

#[derive(Debug)]
pub enum RamSize {
    Mb256,
    Mb512,
    Gb1,
    Gb2,
    Gb4,
    Gb8,
}

impl Display for RamSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RamSize::Mb256 => write!(f, "256MB"),
            RamSize::Mb512 => write!(f, "512MB"),
            RamSize::Gb1 => write!(f, "1GB"),
            RamSize::Gb2 => write!(f, "2GB"),
            RamSize::Gb4 => write!(f, "4GB"),
            RamSize::Gb8 => write!(f, "8GB"),
        }
    }
}

#[derive(Debug)]
pub enum Processor {
    BCM2835,
    BCM2836,
    BCM2837,
    BCM2711,
    BCM2712,
}

#[derive(Debug)]
pub enum Revision {
    Unknown(u32),
    OldStyle {
        code: u32,
        model: Type,
        revision_bcd: u8,
        memory_size: RamSize,
        manufacturer: Manufacturer,
    },
    NewStyle {
        code: u32,
        overvoltage_prohibited: bool,
        otp_program_prohibited: bool,
        otp_reading_prohibited: bool,
        warranty_voided: bool,
        memory_size: RamSize,
        manufacturer: Manufacturer,
        processor: Processor,
        model: Type,
        revision: u8,
    },
}

impl Revision {
    pub fn from_code(code: u32) -> Self {
        use Manufacturer::*;
        use RamSize::*;
        use Type::*;
        match code {
            0x0002 => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x10,
                memory_size: Mb256,
                manufacturer: Egoman,
            },
            0x0003 => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x10,
                memory_size: Mb256,
                manufacturer: Egoman,
            },
            0x0004 => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: SonyUK,
            },
            0x0005 => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: Qisda,
            },
            0x0006 => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: Egoman,
            },
            0x0007 => Self::OldStyle {
                code,
                model: Pi1A,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: Egoman,
            },
            0x0008 => Self::OldStyle {
                code,
                model: Pi1A,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: SonyUK,
            },
            0x0009 => Self::OldStyle {
                code,
                model: Pi1A,
                revision_bcd: 0x20,
                memory_size: Mb256,
                manufacturer: Qisda,
            },
            0x000d => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb512,
                manufacturer: Egoman,
            },
            0x000e => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb512,
                manufacturer: SonyUK,
            },
            0x000f => Self::OldStyle {
                code,
                model: Pi1B,
                revision_bcd: 0x20,
                memory_size: Mb512,
                manufacturer: Egoman,
            },
            0x0010 => Self::OldStyle {
                code,
                model: Pi1BPlus,
                revision_bcd: 0x12,
                memory_size: Mb512,
                manufacturer: SonyUK,
            },
            0x0011 => Self::OldStyle {
                code,
                model: CM1,
                revision_bcd: 0x10,
                memory_size: Mb512,
                manufacturer: SonyUK,
            },
            0x0012 => Self::OldStyle {
                code,
                model: Pi1APlus,
                revision_bcd: 0x11,
                memory_size: Mb256,
                manufacturer: SonyUK,
            },
            0x0013 => Self::OldStyle {
                code,
                model: Pi1BPlus,
                revision_bcd: 0x12,
                memory_size: Mb512,
                manufacturer: Embest,
            },
            0x0014 => Self::OldStyle {
                code,
                model: CM1,
                revision_bcd: 0x10,
                memory_size: Mb512,
                manufacturer: Embest,
            },
            0x0015 => Self::OldStyle {
                code,
                model: Pi1APlus,
                revision_bcd: 0x11,
                memory_size: Mb256,
                manufacturer: Embest,
            },
            x if x >> 23 & 1 != 0 => Self::NewStyle {
                code,
                overvoltage_prohibited: x >> 31 & 1 != 0,
                otp_program_prohibited: x >> 30 & 1 != 0,
                otp_reading_prohibited: x >> 29 & 1 != 0,
                warranty_voided: x >> 25 & 1 != 0,
                memory_size: unsafe { core::mem::transmute((x >> 20) as u8 & 0x7) },
                manufacturer: unsafe { core::mem::transmute((x >> 16) as u8 & 0xF) },
                processor: unsafe { core::mem::transmute((x >> 12) as u8 & 0xF) },
                model: unsafe { core::mem::transmute((x >> 4) as u8 & 0xFF) },
                revision: unsafe { core::mem::transmute(x as u8 & 0xF) },
            },
            _ => Self::Unknown(code),
        }
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Revision::Unknown(code) => write!(f, "Unknown Model Revision [{:#x}]", code),
            Revision::OldStyle {
                code: _,
                model,
                revision_bcd,
                memory_size,
                manufacturer: _,
            } => write!(
                f,
                "Model {} {} {}.{}",
                model,
                memory_size,
                revision_bcd >> 4,
                revision_bcd & 0xf
            ),
            Revision::NewStyle {
                code: _,
                overvoltage_prohibited: _,
                otp_program_prohibited: _,
                otp_reading_prohibited: _,
                warranty_voided: _,
                memory_size,
                manufacturer: _,
                processor: _,
                model,
                revision,
            } => write!(f, "Model {} {} 1.{}", model, memory_size, revision),
        }
    }
}

#[derive(Debug)]
pub struct BoardInfo {
    pub model: u32,
    pub revision: Revision,
    pub serial: u64,
}

pub fn get_arm_memory() -> Option<Memory> {
    let (base_address, size): (u32, u32) = mailbox::simple_single_call(mailbox::Tag::HwGetArmMemory as u32, 8).ok()?;
    Some(Memory {
        base_address: base_address as usize,
        size: ByteSize(size as usize),
    })

}

pub fn get_vc_memory() -> Option<Memory> {
    let (base_address, size): (u32, u32) = mailbox::simple_single_call(mailbox::Tag::HwGetVcMemory as u32, ()).ok()?;
    Some(Memory {
        base_address: base_address as usize,
        size: ByteSize(size as usize),
    })
}

pub fn get_board_info() -> Option<BoardInfo> {
    let mut mb = mailbox::Mailbox::<256>::new();
    mb.push_request_empty(mailbox::Tag::HwGetBoardModel as u32, 4).ok()?;
    mb.push_request_empty(mailbox::Tag::HwGetBoardRevision as u32, 4).ok()?;
    mb.push_request_empty(mailbox::Tag::HwGetBoardSerial as u32, 8).ok()?;
    let mut responses = mb.submit_messages(8).ok()?;

    let model: u32 = responses.next()?.ok()?.try_value_as().copied()?;
    let revision: u32 = responses.next()?.ok()?.try_value_as().copied()?;
    let serial: u64 = responses.next()?.ok()?.try_value_as().copied()?;
    Some(BoardInfo {
        model,
        revision: Revision::from_code(revision),
        serial,
    })
}
