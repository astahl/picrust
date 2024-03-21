use core::{
    num::{NonZeroU16, NonZeroU32},
    usize,
};

use mystd::bit_field;

use super::mmio::PeripheralRegister;

pub const DMA_BASE: usize = 0x7000;
pub const DMA_CHANNEL_SZ: usize = 0x100;

#[derive(Debug)]
pub enum DmaError {
    TransferTooLong,
    AddressNotAligned,
    InvalidPriorityLevel,
    InvalidWaitCycles,
    InvalidPeripheral,
}

pub struct DmaStandardChannel(usize);

pub const DMA_0: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 0 * DMA_CHANNEL_SZ);
pub const DMA_1: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 1 * DMA_CHANNEL_SZ);
pub const DMA_2: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 2 * DMA_CHANNEL_SZ);
pub const DMA_3: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 3 * DMA_CHANNEL_SZ);
pub const DMA_4: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 4 * DMA_CHANNEL_SZ);
pub const DMA_5: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 5 * DMA_CHANNEL_SZ);
pub const DMA_6: DmaStandardChannel = DmaStandardChannel(DMA_BASE + 6 * DMA_CHANNEL_SZ);

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

bit_field!(pub DmaControlAndStatus(u32) {
    0 => active,
    1 => end,
    2 => interrupted,
    3 => data_request,
    4 => paused,
    5 => paused_by_data_request_state,
    6 => waiting_for_outstanding_writes,
    8 => error,
    16:19 => axi_priority_level,
    20:23 => axi_panic_priority_level,
    28 => wait_for_outstanding_writes,
    29 => disable_debug_signal,
    30 => abort,
    31 => reset
});

impl DmaControlAndStatus {
    pub const MAX_PRIORITY_LEVEL: u32 = 0xf;

    #[must_use]
    pub fn clear_interrupt(&self) -> Self {
        // INT is write 1 to clear
        self.interrupted().set()
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
    reserved: [u32; 2],
}

impl core::fmt::Debug for DmaControlBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let is_2d = self.transfer_information._2d_mode().is_set();

        let mut out = f.debug_struct("DmaControlBlock");
        let part_a = out
            .field("transfer_information", &self.transfer_information)
            .field(
                "source_address",
                &format_args!("{:#x}", self.source_address),
            )
            .field(
                "destination_address",
                &format_args!("{:#x}", self.destination_address),
            );
        let part_b = if is_2d {
            part_a.field("transfer_length (2d)", unsafe {
                &self.transfer_length.two_d
            })
        } else {
            part_a.field("transfer_length (linear)", unsafe {
                &self.transfer_length.linear
            })
        };

        part_b
            .field("stride", &self.stride)
            .field(
                "next_control_block_address",
                &format_args!("{:#x}", self.next_control_block_address),
            )
            .field("reserved", &self.reserved)
            .finish()
    }
}

impl DmaControlBlock {
    const MAX_LENGTH: u32 = (1 << 30) - 1;

    pub fn linear_copy(
        transfer_information: DmaTransferInformation,
        source_address: u32,
        destination_address: u32,
        length: u32,
        next_control_block: u32,
    ) -> Self {
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

bit_field!(pub DmaTransferInformation(u32) {
    26 => disable_wide_bursts,
    21:25 => wait_cycles,
    16:20 => peripheral_mapping,
    12:15 => burst_transfer_length,
    11 => src_ignore_reads,
    10 => src_use_data_request,
    9 => src_transfer_width: enum DmaTransferWidth {
        Bit32,
        Bit128,
    },
    8 => src_address_increment,
    7 => dest_ignore_writes,
    6 => dest_use_data_request,
    5 => dest_transfer_width: DmaTransferWidth,
    4 => dest_address_increment,
    3 => wait_for_write_response,
    1 => _2d_mode,
    0 => completion_interrupt
});

impl DmaTransferInformation {
    pub fn wide_copy() -> Self {
        Self::zero()
            .dest_address_increment()
            .set()
            .dest_transfer_width()
            .set_value(DmaTransferWidth::Bit128)
            .src_address_increment()
            .set()
            .src_transfer_width()
            .set_value(DmaTransferWidth::Bit128)
            .burst_transfer_length()
            .set_value(2)
    }

    pub fn narrow_copy() -> Self {
        Self::zero()
            .dest_address_increment()
            .set()
            .dest_transfer_width()
            .set_value(DmaTransferWidth::Bit32)
            .src_address_increment()
            .set()
            .src_transfer_width()
            .set_value(DmaTransferWidth::Bit32)
            .burst_transfer_length()
            .set_value(8)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dma2dStride {
    pub source: i16,
    pub destination: i16,
}

impl Dma2dStride {
    pub fn none() -> Self {
        Self {
            source: 0,
            destination: 0,
        }
    }

    pub fn new(source: i16, destination: i16) -> Self {
        Self {
            destination,
            source,
        }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Clone, Copy, Debug)]
struct DmaTransferLength2d {
    x_byte_len: NonZeroU16,
    y_count: u16,
}

pub union DmaTransferLength {
    linear: NonZeroU32,
    two_d: DmaTransferLength2d,
}

impl DmaTransferLength {
    pub fn new_2d(x_byte_len: NonZeroU16, y_count: u16) -> Self {
        assert!(y_count < 0x4000);
        Self {
            two_d: DmaTransferLength2d {
                x_byte_len,
                y_count,
            },
        }
    }

    pub fn new_linear(byte_len: NonZeroU32) -> Self {
        assert!(byte_len.leading_zeros() > 2);
        Self { linear: byte_len }
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

bit_field!(pub DmaDebug(u32) {
    28 => dma_lite,
    25:27 => version,
    16:24 => dma_state,
    8:15 => dma_id,
    4:8 => outstanding_writes_counter,
    2 => read_error,
    1 => fifo_error,
    0 => read_last_not_set_error
});
