use mystd::bitfield::BitField;

#[no_mangle]
pub extern "C" fn exc_handler(
    exception_type: ExceptionType,
    syndrome: ExceptionSyndrome,
    elr: usize,
    spsr: usize,
    far: usize,
) -> ! {
    use crate::peripherals::uart::Uart0;
    Uart0::init();
    Uart0::puts("Exception Handler!\nException Type: ");
    Uart0::put_uint(exception_type as u64);
    Uart0::puts("\nEC: ");
    Uart0::put_hex(syndrome.exception_class() as u8);
    Uart0::puts("\nISS: ");
    Uart0::put_hex_bytes(&syndrome.instruction_specific_syndrome().to_be_bytes());
    Uart0::puts("\nELR: ");
    Uart0::put_hex_bytes(&elr.to_be_bytes());
    Uart0::puts("\nSPSR: ");
    Uart0::put_hex_bytes(&spsr.to_be_bytes());
    Uart0::puts("\nFAR: ");
    Uart0::put_hex_bytes(&far.to_be_bytes());

    Uart0::putc(b'\n');
    Uart0::put_memory(elr as *const u8, 16);
    loop {
        core::hint::spin_loop();
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum ExceptionType {
    Synchronous = 0_isize,
    IRQ,
    FIQ,
    SError,
}

#[derive(Debug)]
pub enum ExceptionClass {
    Unknown = 0b000000,
    TrappedWFxInstructionExecution = 0b000001,
    Reserved0x02,
    TrappedMCROrMRCAccessCoproc0xF = 0b000011,
    TrappedMCRROrMRRCAccess = 0b000100,
    TrappedMCROrMRCAccessCoproc0xE = 0b000101,
    TrappedLDCOrSTCAccess = 0b000110,
    TrappedFpSMEAdvancedSIMDOrSVE = 0b000111,
    Reserved0x08,
    Reserved0x09,
    TrappedLD64bOrST64bInstruction = 0b001010,
    Reserved0x0c,
    TrappedMRRCAcessCoproc0xE = 0b001100,
    BranchTargetException = 0b001101,
    IllegalExecutionState = 0b001110,
    Reserved0x11,
    TrappedSVCInstructionAArch32 = 0b010001,
    Reserved0x12,
    Reserved0x13,
    Reserved0x14,
    TrappedSVCInstructionAArch64 = 0b010101,
    Reserved0x16,
    Reserved0x17,
    TrappedMSROrMRSOrSystemInstruction = 0b011000,
    TrappedSVEAccess = 0b011001,
    Reserved0x1a,
    ExceptionFromTSTARTInstruction = 0b011011,
    PointerAuthenticationFailure = 0b011100,
    TrappedSMEAccess = 0b011101,
    Reserved0x1e,
    Reserved0x1f,
    InstructionAbortFromLowerEL = 0b100000,
    InstructionAbortFromSameEL = 0b100001,
    ProgramCounterAlignmentFault = 0b100010,
    Reserved0x23,
    DataAbortFromLowerEL = 0b100100,
    DataAbortFromSameEL = 0b100101,
    StackPointerAlignmentFault = 0b100110,
    MemoryOperationException = 0b100111,
    TrappedFloatingPointAArch32 = 0b101000,
    Reserved0x29,
    Reserved0x2a,
    Reserved0x2b,
    TrappedFloatingPointAArch64 = 0b101100,
    Reserved0x2d,
    Reserved0x2e,
    SError = 0b101111,
    BreakpointFromLowerEL = 0b110000,
    BreakpointFromSameEL = 0b110001,
    SoftwareStepFromLowerEL = 0b110010,
    SoftwareStepFromSameEL = 0b110011,
    WatchpointFromLowerEL = 0b110100,
    WatchpointFromSameEL = 0b110101,
    Reserved0x36,
    Reserved0x37,
    BKPTInstructionAArch32 = 0b111000,
    Reserved0x39,
    Reserved0x3a,
    Reserved0x3b,
    BRKInstructionAArch64 = 0b111100,
    Reserved0x3d,
    Reserved0x3e,
    Reserved0x3f,
}

pub enum InstructionLength {
    Trapped16bitInstruction,
    Trapped32bitInstruction,
}

#[repr(C)]
pub struct ExceptionSyndrome(BitField<usize>);

impl ExceptionSyndrome {
    pub fn exception_class(&self) -> ExceptionClass {
        unsafe { core::mem::transmute(self.0.field(26, 6) as u8) }
    }

    pub fn instruction_length(&self) -> InstructionLength {
        unsafe { core::mem::transmute(self.0.bit_value(25) as u8) }
    }

    pub fn instruction_specific_syndrome(&self) -> u32 {
        self.0.field(0, 13) as u32
    }

    pub fn raw_value(&self) -> usize {
        self.0 .0
    }
}
