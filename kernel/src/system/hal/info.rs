use core::{fmt::{Debug, Display, Write}, ptr::null};

use crate::{peripherals::mailbox, system::peripherals};


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

pub fn get_arm_memory() -> Option<MemoryBlock> {
    let (base_address, size): (u32, u32) =
        mailbox::simple_single_call(mailbox::Tag::HwGetArmMemory as u32, 8).ok()?;
    Some(MemoryBlock::from_address_and_size(base_address as usize, size as usize))
}

pub fn get_vc_memory() -> Option<MemoryBlock> {
    let (base_address, size): (u32, u32) =
        mailbox::simple_single_call(mailbox::Tag::HwGetVcMemory as u32, ()).ok()?;
    Some(MemoryBlock::from_address_and_size(base_address as usize, size as usize))
}

pub fn get_board_info() -> Option<BoardInfo> {
    let mut mb = mailbox::Mailbox::<256>::new();
    mb.push_request_empty(mailbox::Tag::HwGetBoardModel as u32, 4)
        .ok()?;
    mb.push_request_empty(mailbox::Tag::HwGetBoardRevision as u32, 4)
        .ok()?;
    mb.push_request_empty(mailbox::Tag::HwGetBoardSerial as u32, 8)
        .ok()?;
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


extern "C" {
    static __main_stack: u8;
    static __kernel_start: u8;
    static __kernel_txt_start: u8;
    static __kernel_txt_end: u8;
    static __rodata_start: u8;
    static __font_start: u8;
    static __font_end: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
    static __kernel_end: u8;
    static __free_memory_start: u8;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryBlock(*const u8, *const u8);

impl MemoryBlock {
    pub fn bottom(self) -> *const u8 {
        self.0
    }

    pub fn top(self) -> *const u8 {
        self.1
    }
}

impl core::fmt::Debug for MemoryBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{:#x} - {:#x}]", self.0 as usize, self.1 as usize)?;
        if f.alternate() {
            write!(f, "({})", mystd::format::ByteValue(self.byte_size()))?;
        }
        Ok(())
    }
}

impl core::fmt::Display for MemoryBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl MemoryBlock{
    pub fn from_symbols<T>(start: &T, end: &T) -> Self {
        let st = core::ptr::addr_of!(*start).cast::<u8>();
        let end = core::ptr::addr_of!(*end).cast::<u8>();
        Self(st.min(end), st.max(end))
    }

    pub fn from_zero_to_symbol<T>(end: &T) -> Self {
        let st = null::<u8>();
        let end = core::ptr::addr_of!(*end).cast::<u8>();
        Self(st, end)
    }

    pub const fn from<T>(entity: &T) -> Self {
        Self::from_start_and_count(entity, 1)
    }

    pub const fn from_address_and_size(address: usize, size: usize) -> Self {
        Self(address as *const u8, (address + size) as *const u8)
    }

    pub const fn from_start_and_count<T>(start: &T, count: usize) -> Self {
        Self(core::ptr::addr_of!(*start).cast(), core::ptr::addr_of!(*start).wrapping_add(count).cast())
    }

    pub fn byte_size(&self) -> usize {
        (self.0 as usize).abs_diff(self.1 as usize)
    }
}

pub struct MemoryMap();
impl MemoryMap {
    pub fn main_stack() -> MemoryBlock {
        unsafe { MemoryBlock::from_zero_to_symbol(&__main_stack) }
    }
}

impl core::fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe{
        let stack = Self::main_stack();
        let kernel_text = MemoryBlock::from_symbols(&__kernel_txt_start, &__kernel_txt_end);
        let kernel = MemoryBlock::from_symbols(&__kernel_start, &__kernel_end);
        let rodata = MemoryBlock::from_symbols(&__rodata_start, &__rodata_end);
        let font = MemoryBlock::from_symbols(&__font_start, &__font_end);
        let data = MemoryBlock::from_symbols(&__data_start, &__data_end);
        let bss = MemoryBlock::from_symbols(&__bss_start, &__bss_end);
        let arm_ram = self::get_arm_memory().ok_or(core::fmt::Error)?;
        let heap = MemoryBlock::from_symbols(&__kernel_end, &*arm_ram.1);
        let vc_ram = self::get_vc_memory().ok_or(core::fmt::Error)?;
        let peripherals = MemoryBlock::from_address_and_size(peripherals::BCM_HOST.peripheral_address, peripherals::BCM_HOST.peripheral_size);
        f.debug_struct("MemoryMap")
            .field("Stack", &stack)
            .field("Kernel", &kernel)
            .field("Kernel Code", &kernel_text)
            .field("Read-Only Data Segment", &rodata)
            .field("Font", &font)
            .field("Data Segment", &data)
            .field("BSS Segment", &bss)
            .field("Heap", &heap)
            .field("ARM", &arm_ram)
            .field("VC", &vc_ram)
            .field("Peripherals", &peripherals)
            .field("Peripherals", &peripherals::PeripheralMap())
            .finish()
        }
    }
}

