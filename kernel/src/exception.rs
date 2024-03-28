use core::{arch::global_asm, fmt::Debug};

use mystd::{bit_field, bitfield::BitField};

use crate::system::peripherals::uart::UART_0;

#[no_mangle]
pub extern "C" fn exc_handler(
    exception_data: AuxExceptionData,
    syndrome: ExceptionSyndrome,
    elr: usize,
    spsr: usize,
    far: usize,
) -> ! {
    use mystd::io::Write;
    let mut uart = UART_0;
    uart.init();
    writeln!(&mut uart, "Exception Handler!").unwrap_or_default();
    writeln!(&mut uart, "Exception Data: {:?}", exception_data).unwrap_or_default();
    writeln!(&mut uart, "{:#?}", syndrome).unwrap_or_default();
    writeln!(
        &mut uart,
        "ELR:  {:0width$x}",
        elr,
        width = core::mem::size_of::<usize>() * 2
    )
    .unwrap_or_default();
    writeln!(
        &mut uart,
        "SPSR: {:0width$x}",
        spsr,
        width = core::mem::size_of::<usize>() * 2
    )
    .unwrap_or_default();
    writeln!(
        &mut uart,
        "FAR:  {:0width$x}",
        far,
        width = core::mem::size_of::<usize>() * 2
    )
    .unwrap_or_default();

    // Uart0::putc(b'\n');
    // Uart0::put_memory(elr as *const u8, 16);
    panic!("EXCEPTION");
    loop {
        core::hint::spin_loop();
    }
}


bit_field!(pub AuxExceptionData (u64) {
    3:2 => origin: enum ExceptionOrigin {
        CurrentElSpEl0 = 0,
        CurrentElSpElx = 1,
        LowerElAarch64 = 2,
        LowerElAarch32 = 3
    },
    1:0 => exception_type: enum ExceptionType {
        Synchronous = 0,
        IRQ = 1,
        FIQ = 2,
        SError = 3,
    },
});


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

#[derive(Debug)]
pub enum InstructionLength {
    Trapped16bitInstruction,
    Trapped32bitInstruction,
}

#[repr(C)]
pub struct ExceptionSyndrome(BitField<usize>);

impl Debug for ExceptionSyndrome {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Exception Syndrome")
            .field("Bits", &format_args!("{:#?}", self.0))
            .field("Exception Class", &self.exception_class())
            .field("Instruction Length", &self.instruction_length())
            .field(
                "Instruction Specific Syndrome",
                &self.instruction_specific_syndrome(),
            )
            .finish()
    }
}

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
}


global_asm!(
    r#"
    .section ".text.vector"
.macro push_registers
	sub 	sp, sp, #256
	stp 	x0, x1, [sp, #16 * 0]
	stp 	x2, x3, [sp, #16 * 1]
	stp	x4, x5, [sp, #16 * 2]
	stp	x6, x7, [sp, #16 * 3]
	stp	x8, x9, [sp, #16 * 4]
	stp	x10, x11, [sp, #16 * 5]
	stp	x12, x13, [sp, #16 * 6]
	stp	x14, x15, [sp, #16 * 7]
	stp	x16, x17, [sp, #16 * 8]
	stp	x18, x19, [sp, #16 * 9]
	stp	x20, x21, [sp, #16 * 10]
	stp	x22, x23, [sp, #16 * 11]
	stp	x24, x25, [sp, #16 * 12]
	stp	x26, x27, [sp, #16 * 13]
	stp	x28, x29, [sp, #16 * 14]
	str	x30, [sp, #16 * 15] 
.endm

.macro pop_registers
	ldp	x0, x1, [sp, #16 * 0]
	ldp	x2, x3, [sp, #16 * 1]
	ldp	x4, x5, [sp, #16 * 2]
	ldp	x6, x7, [sp, #16 * 3]
	ldp	x8, x9, [sp, #16 * 4]
	ldp	x10, x11, [sp, #16 * 5]
	ldp	x12, x13, [sp, #16 * 6]
	ldp	x14, x15, [sp, #16 * 7]
	ldp	x16, x17, [sp, #16 * 8]
	ldp	x18, x19, [sp, #16 * 9]
	ldp	x20, x21, [sp, #16 * 10]
	ldp	x22, x23, [sp, #16 * 11]
	ldp	x24, x25, [sp, #16 * 12]
	ldp	x26, x27, [sp, #16 * 13]
	ldp	x28, x29, [sp, #16 * 14]
	ldr	x30, [sp, #16 * 15] 
	add	sp, sp, #256
.endm
    .global _vectors_el1
    // important: code has to be properly aligned to 2^11 = 0x800 = 2048 bytes
    _vectors_el1:

    // Origin: Current Exception level with SP_EL0.

    // synchronous
    .align  7 // alignment of 128 bytes
        push_registers
        mov     x0, #0
        b       _handle_and_return_el1
    
    // IRQ or vIRQ
    .align  7
        push_registers
        mov     x0, #1
        b       _handle_and_return_el1

    // FIQ or vFIQ
    .align  7
        push_registers
        mov     x0, #2
        b       _handle_and_return_el1
    
    // SError or vSError
    .align  7
        push_registers
        mov     x0, #3
        b       _handle_and_return_el1


    // Origin: Current Exception level with SP_ELx, x > 0.

    // synchronous 0x200
    .align  7 // alignment of 128 bytes
        push_registers
        mov     x0, #4
        b       _handle_and_return_el1
    
    // IRQ or vIRQ 0x280
    .align  7
        push_registers
        mov     x0, #5
        b       _handle_and_return_el1

    // FIQ or vFIQ 0x300
    .align  7
        push_registers
        mov     x0, #6
        b       _handle_and_return_el1
    
    // SError or vSError 0x380
    .align  7
        push_registers
        mov     x0, #7
        b       _handle_and_return_el1

    
    // Origin: Lower Exception level, where the implemented level immediately lower than the target level is using AArch64.

    // synchronous 0x400
    .align  7 // alignment of 128 bytes
        push_registers
        mov     x0, #8
        b       _handle_and_return_el1
    
    // IRQ or vIRQ 0x480
    .align  7
        push_registers
        mov     x0, #9
        b       _handle_and_return_el1

    // FIQ or vFIQ 0x500
    .align  7
        push_registers
        mov     x0, #10
        b       _handle_and_return_el1
    
    // SError or vSError 0x580
    .align  7
        push_registers
        mov     x0, #11
        b       _handle_and_return_el1

    // Origin: Lower Exception level, where the implemented level immediately lower than the target level is using AArch64.

    // synchronous 0x600
    .align  7 // alignment of 128 bytes
        push_registers
        mov     x0, #12
        b       _handle_and_return_el1
    
    // IRQ or vIRQ 0x680
    .align  7
        push_registers
        mov     x0, #13
        b       _handle_and_return_el1

    // FIQ or vFIQ 0x700
    .align  7
        push_registers
        mov     x0, #14
        b       _handle_and_return_el1
    
    // SError or vSError 0x780
    .align  7
        push_registers
        mov     x0, #15
        b       _handle_and_return_el1

    _handle_and_return_el1:
        mrs     x1, esr_el1
        mrs     x2, elr_el1
        mrs     x3, spsr_el1
        mrs     x4, far_el1
        bl      exc_handler
        pop_registers
        eret
    "#
);
