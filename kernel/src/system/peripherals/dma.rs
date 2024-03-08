use core::{num::{NonZeroU16, NonZeroU32}, usize};

use mystd::bitfield::BitField;

use super::mmio::PeripheralRegister;

#[derive(Debug)]
pub enum DmaError {
    TransferTooLong,
    AddressNotAligned,
    InvalidPriorityLevel,
    InvalidWaitCycles,
    InvalidPeripheral,
}

pub struct DmaStandardChannel(usize);

pub const DMA_0: DmaStandardChannel = DmaStandardChannel(0x7000);
pub const DMA_1: DmaStandardChannel = DmaStandardChannel(0x7100);
pub const DMA_2: DmaStandardChannel = DmaStandardChannel(0x7200);
pub const DMA_3: DmaStandardChannel = DmaStandardChannel(0x7300);
pub const DMA_4: DmaStandardChannel = DmaStandardChannel(0x7400);
pub const DMA_5: DmaStandardChannel = DmaStandardChannel(0x7500);
pub const DMA_6: DmaStandardChannel = DmaStandardChannel(0x7600);

type ControlAndStatusReg = PeripheralRegister<0x00, DmaControlAndStatus>; 
type ControlBlockAddressReg = PeripheralRegister<0x04, u32>; 
type TransferInformationReg = PeripheralRegister<0x08, DmaTransferInformation>;
type SourceAddressReg = PeripheralRegister<0x0c, u32>;
type DestinationAddressReg = PeripheralRegister<0x10, u32>;
type TransferLengthReg = PeripheralRegister<0x14, DmaTransferLength>;
type StrideReg = PeripheralRegister<0x18, Dma2dStride>;
type NextControlBlockAddressReg = PeripheralRegister<0x1c, u32>;
type DebugReg = PeripheralRegister<0x20, DmaDebug>;

impl DmaStandardChannel {
    pub fn control_and_status(&self) -> DmaControlAndStatus {
        ControlAndStatusReg::at(self.0).read()
    }

    pub fn set_control_and_status(&self, value: DmaControlAndStatus) {
        ControlAndStatusReg::at(self.0).write(value)
    }

    pub fn set_control_block_address(&self, value: u32) {
        ControlBlockAddressReg::at(self.0).write(value)
    }

    pub fn control_block_address(&self) -> u32 {
        ControlBlockAddressReg::at(self.0).read()
    }

    pub fn debug(&self) -> DmaDebug {
        DebugReg::at(self.0).read()
    }
}

#[derive(Clone, Copy)]
pub struct DmaControlAndStatus(BitField<u32>);

impl DmaControlAndStatus{
    pub const MAX_PRIORITY_LEVEL: u32 = 0xf;

    pub fn is_active(&self) -> bool {
        self.0.bit_test(0)
    }

    #[must_use]
    pub fn with_active_set(&self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    #[must_use]
    pub fn with_active_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }

    pub fn is_end(&self) -> bool {
        self.0.bit_test(1)
    }

    #[must_use]
    pub fn with_end_cleared(&self) -> Self {
        // END is write 1 to clear
        Self(self.0.with_bit_set(1))
    }

    pub fn is_interrupted(&self) -> bool {
        self.0.bit_test(2)
    }

    #[must_use]
    pub fn with_interrupted_cleared(&self) -> Self {
        // INT is write 1 to clear
        Self(self.0.with_bit_set(2))
    }

    pub fn is_data_request(&self) -> bool {
        self.0.bit_test(3)
    }

    pub fn is_paused(&self) -> bool {
        self.0.bit_test(4)
    }

    pub fn is_paused_by_data_request_state(&self) -> bool {
        self.0.bit_test(5)
    }

    pub fn is_waiting_for_outstanding_writes(&self) -> bool {
        self.0.bit_test(6)
    }

    pub fn is_error(&self) -> bool {
        self.0.bit_test(8)
    }

    pub fn axi_priority_level(&self) -> u32 {
        self.0.field(16, 4)
    }

    #[must_use]
    pub fn with_axi_priority_level(&self, level: u32) -> Result<Self, DmaError> {
        if level > Self::MAX_PRIORITY_LEVEL {
            Err(DmaError::InvalidPriorityLevel)
        } else {
            Ok(Self(self.0.with_field_set(16, 4, level)))
        }
    }

    pub fn axi_panic_priority_level(&self) -> u32 {
        self.0.field(20, 4)
    }

    #[must_use]
    pub fn with_axi_panic_priority_level(&self, level: u32) -> Result<Self, DmaError> {
        if level > Self::MAX_PRIORITY_LEVEL {
            Err(DmaError::InvalidPriorityLevel)
        } else {
            Ok(Self(self.0.with_field_set(20, 4, level)))
        }
    }

    pub fn will_wait_for_outstanding_writes(&self) -> bool {
        self.0.bit_test(28)
    }

    #[must_use]
    pub fn with_wait_for_outstanding_writes_set(&self) -> Self {
        Self(self.0.with_bit_set(28))
    }

    #[must_use]
    pub fn with_wait_for_outstanding_writes_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(28))
    }

    #[must_use]
    pub fn with_debug_pause_signal_disabled(&self) -> Self {
        Self(self.0.with_bit_set(29))
    }

    #[must_use]
    pub fn with_debug_pause_signal_enabled(&self) -> Self {
        Self(self.0.with_bit_cleared(29))
    }

    pub fn is_debug_pause_signal_disabled(&self) -> bool {
        self.0.bit_test(29)
    }

    #[must_use]
    pub fn with_abort_set(&self) -> Self {
        Self(self.0.with_bit_set(30))
    }

    pub fn is_aborting(&self) -> bool {
        self.0.bit_test(30)
    }

    #[must_use]
    pub fn with_reset_set(&mut self) -> Self {
        Self(self.0.with_bit_set(31))
    }

    pub fn is_resetting(&self) -> bool {
        self.0.bit_test(31)
    }
}

impl core::fmt::Debug for DmaControlAndStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DmaControlAndStatus")
            .field("ACTIVE", &self.is_active())
            .field("END", &self.is_end())
            .field("INT", &self.is_interrupted())
            .field("PAUSED", &self.is_paused())
            .field("DREQ_STOPS_DMA", &self.is_paused_by_data_request_state())
            .field("WAITING_FOR_OUTSTANDING_WRITE", &self.is_waiting_for_outstanding_writes())
            .field("ERROR", &self.is_error())
            .field("PRIORITY", &self.axi_priority_level())
            .field("PANIC_PRIORITY", &self.axi_panic_priority_level())
            .field("DISDEBUG", &self.is_debug_pause_signal_disabled())
            .field("ABORT", &self.is_aborting())
            .field("RESET", &self.is_resetting())
            .finish()
    }
}


#[repr(C, align(32))]
pub struct DmaControlBlock {
    transfer_information: DmaTransferInformation,
    source_address: u32,
    destination_address: u32,
    transfer_length: DmaTransferLength,
    stride: Dma2dStride,
    next_control_block_address: u32,
    reserved: [u32;2]
}

impl DmaControlBlock {
    const MAX_LENGTH: u32 = (1 << 30) - 1;

    pub fn linear_copy(transfer_information: DmaTransferInformation, source_address: u32, destination_address: u32, length: u32, next_control_block: u32) -> Self {
        assert!(length <= Self::MAX_LENGTH);
        assert_eq!(0, length % 4);
        Self {
            transfer_information,
            source_address,
            destination_address,
            transfer_length: DmaTransferLength::new_linear(length.try_into().unwrap()),
            stride: Dma2dStride::none(),
            next_control_block_address: next_control_block,
            reserved: [0, 0],
        }
    }
}


pub enum DmaTransferWidth {
    Bit32,
    Bit128
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DmaTransferInformation(BitField<u32>);

impl DmaTransferInformation {
    pub fn wide_copy() -> Self {
        *Self::new().dest_address_increment(true)
            .dest_transfer_width(DmaTransferWidth::Bit128)
            .src_address_increment(true)
            .src_transfer_width(DmaTransferWidth::Bit128)
            .burst_transfer_length(2)
    }

    pub fn narrow_copy() -> Self {
        *Self::new().dest_address_increment(true)
            .dest_transfer_width(DmaTransferWidth::Bit32)
            .src_address_increment(true)
            .src_transfer_width(DmaTransferWidth::Bit32)
            .burst_transfer_length(8)
    }

    pub const fn new() -> Self {
        Self(BitField(0))
    }

    pub fn value(&self) -> u32 {
        self.0.0
    }

    pub fn disable_wide_bursts(&mut self) -> &mut Self {
        self.0.bit_set(26);
        self
    }

    pub fn add_wait_cycles(&mut self, cycles: u32) -> &mut Self {
        debug_assert!(cycles < 32);
        self.0.field_set(21, 5, cycles);
        self
    }

    pub fn peripheral_mapping(&mut self, peripheral_num: u32) -> &mut Self {
        debug_assert!(peripheral_num < 32);
        self.0.field_set(16, 5, peripheral_num);
        self
    }

    pub fn burst_transfer_length(&mut self, word_count: u32) -> &mut Self {
        debug_assert!(word_count < 32);
        self.0.field_set(12, 4, word_count);
        self
    }

    pub fn src_ignore_reads(&mut self) -> &mut Self {
        self.0.bit_set(11);
        self
    }

    pub fn src_use_dreq(&mut self) -> &mut Self {
        self.0.bit_set(10);
        self
    }

    pub fn src_transfer_width(&mut self, width: DmaTransferWidth) -> &mut Self {
        match width {
            DmaTransferWidth::Bit32 => self.0.bit_clear(9),
            DmaTransferWidth::Bit128 => self.0.bit_set(9),
        }
        self
    }

    pub fn src_address_increment(&mut self, enable: bool) -> &mut Self {
        if enable { 
            self.0.bit_set(8);
        } else {
            self.0.bit_clear(8);
        }
        self
    }

    pub fn dest_ignore_writes(&mut self) -> &mut Self {
        self.0.bit_set(7);
        self
    }

    pub fn dest_use_dreq(&mut self) -> &mut Self {
        self.0.bit_set(6);
        self
    }

    pub fn dest_transfer_width(&mut self, width: DmaTransferWidth) -> &mut Self {
        match width {
            DmaTransferWidth::Bit32 => self.0.bit_clear(5),
            DmaTransferWidth::Bit128 => self.0.bit_set(5),
        }
        self
    }

    pub fn dest_address_increment(&mut self, enable: bool) -> &mut Self {
        if enable { 
            self.0.bit_set(4);
        } else {
            self.0.bit_clear(4);
        }
        self
    }

    pub fn wait_for_write_response(&mut self, enable: bool) -> &mut Self {
        if enable { 
            self.0.bit_set(3);
        } else {
            self.0.bit_clear(3);
        }
        self
    }

    pub fn set_2d_mode(&mut self, enable: bool) -> &mut Self {
        if enable { 
            self.0.bit_set(1);
        } else {
            self.0.bit_clear(1);
        }
        self
    }

    pub fn set_completion_interrupt(&mut self, enable: bool) -> &mut Self {
        if enable { 
            self.0.bit_set(0);
        } else {
            self.0.bit_clear(0);
        }
        self
    }
}

#[derive(Clone, Copy)]
pub struct Dma2dStride{
    pub source: i16,
    pub destination: i16
}

impl Dma2dStride {
    pub fn none() -> Self {
        Self { source: 0, destination: 0 }
    }

    pub fn new(source: i16, destination: i16) -> Self {
        Self{ destination, source }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Clone, Copy)]
struct DmaTransferLength2d {
    x_byte_len: NonZeroU16, y_count: u16
}

pub union DmaTransferLength {
    linear: NonZeroU32,
    two_d: DmaTransferLength2d
}

impl DmaTransferLength {
    pub fn new_2d(x_byte_len: NonZeroU16, y_count: u16) -> Self {
        assert!(y_count < 0x4000);
        Self { two_d: DmaTransferLength2d { x_byte_len, y_count } }
    }

    pub fn new_linear(byte_len: NonZeroU32) -> Self {
        assert!(byte_len.leading_zeros() > 2);
        Self{ linear: byte_len }
    }

    pub fn linear(self) -> NonZeroU32 {
        unsafe { self.linear }
    }

    pub fn x_byte_len(self) -> NonZeroU16 {
        unsafe { self.two_d.x_byte_len }
    }

    pub fn y_count(self) -> u16 {
        unsafe { self.two_d.y_count }
    }
}

#[derive(Clone, Copy)]
pub struct DmaDebug (BitField<u32>);

impl DmaDebug {
    pub fn is_dma_lite(&self) -> bool {
        self.0.bit_test(28)
    }

    pub fn version(&self) -> u32 {
        self.0.field(25, 3) 
    }

    pub fn dma_state(&self) -> u32 {
        self.0.field(16, 9)
    }

    pub fn dma_id(&self) -> u32 {
        self.0.field(8, 8)
    }

    pub fn outstanding_writes_counter(&self) -> u32 {
        self.0.field(4, 4)
    }

    pub fn is_read_error(&self) -> bool {
        self.0.bit_test(2)
    }

    #[must_use]
    pub fn with_read_error_clear(&self) -> Self {
        Self(self.0.with_bit_set(2))
    }

    pub fn is_fifo_error(&self) -> bool {
        self.0.bit_test(1)
    }

    #[must_use]
    pub fn with_fifo_error_clear(&self) -> Self {
        Self(self.0.with_bit_set(1))
    }

    pub fn is_read_last_not_set_error(&self) -> bool {
        self.0.bit_test(0)
    }

    #[must_use]
    pub fn with_read_last_not_set_error_clear(&self) -> Self {
        Self(self.0.with_bit_set(0))
    }
}

impl core::fmt::Debug for DmaDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DmaDebug")
            .field("LITE", &self.is_dma_lite())
            .field("VERSION", &self.version())
            .field("DMA_STATE", &self.dma_state())
            .field("DMA_ID", &self.dma_id())
            .field("OUTSTANDING_WRITES", &self.outstanding_writes_counter())
            .field("READ_ERROR", &self.is_read_error())
            .field("FIFO_ERROR", &self.is_fifo_error())
            .field("READ_LAST_NOT_SET_ERROR", &self.is_read_last_not_set_error())
            .finish()
    }
}