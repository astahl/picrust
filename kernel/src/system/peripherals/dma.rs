use core::{
    num::{NonZeroU16, NonZeroU32}, ops::Sub, usize
};

use mystd::{bit_field, byte_value::ByteValue, slice::slice2d::{traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d, Slice2d}};

use crate::{print_log, println_debug, println_log};

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

pub const DMA_0: DmaStandardChannel = DmaStandardChannel(DMA_BASE);
pub const DMA_1: DmaStandardChannel = DmaStandardChannel(DMA_BASE + DMA_CHANNEL_SZ);
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
    pub fn control_and_status(&self) -> ControlAndStatusReg {
        ControlAndStatusReg::at(self.0)
    }

    pub fn control_block_address(&self) -> ControlBlockAddressReg {
        ControlBlockAddressReg::at(self.0)
    }

    /// # DMA Next Control Block Address
    /// The value loaded into this register can be overwritten so that the linked list of Control Block data structures can be altered. However it is only safe to do this when the DMA is paused. 
    /// The address must be 256-bit aligned and so the bottom 5 bits cannot be set and will read back as zero.
    #[doc(alias = "NEXTCONBK")]
    pub fn next_control_block_address(&self) -> NextControlBlockAddressReg {
        NextControlBlockAddressReg::at(self.0)
    }

    pub fn transfer_length(&self) -> TransferLengthReg {
        TransferLengthReg::at(self.0)
    }

    /// DMA Debug register
    pub fn debug(&self) -> DebugReg {
        DebugReg::at(self.0)
    }

    pub fn start_transfer(&self, control_block: &DmaControlBlock) {
        self.control_block_address().write(core::ptr::addr_of!(*control_block) as u32);
        self
            .control_and_status()
            .update(|status| 
                status
                    //.wait_for_outstanding_writes().set()
                    .active().set()
                );
    }

    pub fn wait_for_idle(&self) {
        while self.control_and_status().read().active().is_set() {
            core::hint::spin_loop();
        }
    }

    pub fn wait_for_end(&self) {
        while self.control_and_status().read().end().is_clear() {
            core::hint::spin_loop();
        }
        self.control_and_status().update(|cs| cs.end().set());
    }
}

bit_field!(pub DmaControlAndStatus(u32) {
    /// # Activate the DMA (RW)
    /// This bit enables the DMA. The DMA will start if this bit is set and the CB_ADDR is non zero. The DMA transfer can be paused and resumed by clearing, then setting it again.
    /// This bit is automatically cleared at the end of the complete DMA transfer, i.e. after a NEXTCONBK = 0x0000_0000 has been loaded.
    0 => active,
    
    /// # DMA End Flag (W1C)
    /// Set when the transfer described by the current Control Block is complete. Write 1 to clear.
    1 => end,

    /// # Interrupt Status (W1C)
    /// This is set when the transfer for the CB ends and INTEN is set to 1. Once set it must be manually cleared down, even if the next CB has INTEN = 0.
    /// Write 1 to clear.
    //#[doc(alias = "INT")]
    2 => interrupted,

    /// # DREQ State (RO)
    /// Indicates the state of the selected DREQ (Data Request) signal, i.e. the DREQ selected by the PERMAP field of the transfer info.
    /// * 1 = Requesting data. This will only be valid once the DMA has started and the PERMAP field has been loaded from the CB. It will remain valid, indicating the selected DREQ signal, until a new CB is loaded. If PERMAP is set to zero (un-paced transfer) then this bit will read back as 1.
    /// * 0 = No data request.
    3 => data_request,

    /// # DMA Paused State (RO)
    /// Indicates if the DMA is currently paused and not transferring data. This will occur if: the active bit has been cleared, the DMA is currently executing wait cycles, the debug_pause signal has been set by the debug block, or the number of outstanding writes has exceeded the max count. 
    /// * 1 = DMA channel is paused.
    /// * 0 = DMA channel is running.
    4 => paused,

    /// # DMA Paused by DREQ State
    /// Indicates if the DMA is currently paused and not transferring data due to the DREQ being inactive. 
    /// * 1 = DMA channel is paused.
    /// * 0 = DMA channel is running.
    5 => paused_by_data_request_state,

    /// # DMA is Waiting for the Last Write to be Received (RO)
    /// Indicates if the DMA is currently waiting for any outstanding writes to be received, and is not transferring data.
    /// * 1 = DMA channel is waiting.
    6 => waiting_for_outstanding_writes,

    /// # DMA Error (RO)
    /// Indicates if the DMA has detected an error. The error flags are available in the debug register, and have to be cleared by writing to that register.
    /// * 1 = DMA channel has an error flag set.
    /// * 0 = DMA channel is OK.
    8 => error,

    /// # AXI Priority Level (RW)
    /// Sets the priority of normal AXI bus transactions. This value is used when the panic bit of the selected peripheral channel is zero.
    /// * Zero is the lowest priority.
    16:19 => axi_priority_level,

    /// # AXI Panic Priority Level (RW)
    /// Sets the priority of panicking AXI bus transactions. This value is used when the panic bit of the selected peripheral channel is 1.
    /// * Zero is the lowest priority.
    20:23 => axi_panic_priority_level,

    /// # Wait for outstanding writes (RW)
    /// When set to 1, the DMA will keep a tally of the AXI writes going out and the write responses coming in. 
    /// At the very end of the current DMA transfer it will wait until the last outstanding write response has 
    /// been received before indicating the transfer is complete. Whilst waiting it will load the next CB 
    /// address (but will not fetch the CB), clear the active flag (if the next CB address = zero), and it will 
    /// defer setting the END flag or the INT flag until the last outstanding write response has been received.
    /// In this mode, the DMA will pause if it has more than 13 outstanding writes at any one time.
    28 => wait_for_outstanding_writes,

    /// # Disable debug pause signal (RW)
    /// When set to 1, the DMA will not stop when the debug pause signal is asserted.
    29 => disable_debug_signal,

    /// # Abort DMA (W1SC)
    /// Writing a 1 to this bit will abort the current DMA CB. The DMA will load the next CB and attempt to continue. 
    /// The bit cannot be read, and will self clear.
    30 => abort,

    /// # DMA Channel Reset (W1SC)
    /// Writing a 1 to this bit will reset the DMA. The bit cannot be read, and will self clear.
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
    const MAX_HEIGHT: u16 = (1 << 14) - 1;

    pub fn new_linear_copy(
        transfer_information: DmaTransferInformation,
        source_address: u32,
        destination_address: u32,
        length: u32,
        next_control_block_address: u32,
    ) -> Self {
        assert!(length <= Self::MAX_LENGTH, "Can't copy more than {} bytes in one transfer, length={}", Self::MAX_LENGTH, length);
        assert_eq!(0, length % 4);
        Self {
            transfer_information,
            source_address,
            destination_address,
            transfer_length: DmaTransferLength::new_linear(length.try_into().unwrap()),
            stride: Dma2dStride::none(),
            next_control_block_address,
            reserved: [0, 0],
        }
    }

    pub fn new_2d_copy(
        transfer_information: DmaTransferInformation,
        source_address: u32,
        destination_address: u32,
        y_count: u16,
        x_byte_len: NonZeroU16,
        src_stride: i16,
        dst_stride: i16,
        next_control_block_address: u32) -> Self {
            assert!(y_count <= Self::MAX_HEIGHT, "Can't copy more than {} lines in one transfer, y_count={}", Self::MAX_HEIGHT, y_count);
            debug_assert!(transfer_information._2d_mode().is_set(), "Must set 2d mode in transfer information");
            Self {
                transfer_information,
                source_address,
                destination_address,
                transfer_length: DmaTransferLength::new_2d(x_byte_len, y_count),
                stride: Dma2dStride::new(src_stride, dst_stride),
                next_control_block_address,
                reserved: [0,0],
            }
        }
    
    pub fn copy_slice<T>(src: &[T], dst: &mut [T]) -> Self {
        assert_eq!(src.len(), dst.len(), "Source and destination must be the same length");
        Self::new_linear_copy(
            DmaTransferInformation::wide_copy(), 
            src.as_ptr() as u32, 
            dst.as_mut_ptr() as u32, 
            (core::mem::size_of_val(src)) as u32, 
            0)
    }

    pub fn copy_slice2d<T>(src: &Slice2d<T>, dst: &mut MutSlice2d<T>) -> Self {   
        let src_stride: i16 = ((src.pitch() - src.width()) * core::mem::size_of::<T>()) as i16;
        let dst_stride: i16 = ((dst.pitch() - dst.width()) * core::mem::size_of::<T>()) as i16;

        if src_stride == 0 && dst_stride == 0 {
            // if there is no stride to handle, we can just use the linear copy instead
            return Self::copy_slice(src.buf_slice(), dst.buf_mut_slice())
        }
        assert_eq!(src.width(), dst.width(), "Source and destination must be the same width");
        assert_eq!(src.height(), dst.height(), "Source and destination must be the same height");
        assert!(src.height() <= (u16::MAX >> 2) as usize, "Height is too tall for one-shot copy");

        let x_byte_len = src.width() * core::mem::size_of::<T>();
        assert!(x_byte_len <= u16::MAX as usize, "Width is too wide for one-shot copy");

        
        Self::new_2d_copy(
            DmaTransferInformation::wide_copy()._2d_mode().set(), 
            src.as_ptr() as u32, 
            dst.as_mut_ptr() as u32, 
            src.height() as u16, 
            NonZeroU16::new(x_byte_len as u16).expect("Width should not be zero"), 
            src_stride, dst_stride, 0)
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

#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DmaTransferLength2d {
    x_byte_len: NonZeroU16,
    y_count: u16,
}

#[repr(C)]
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


pub fn dma_copy_slice<T>(src: &[T], dst: &mut [T]) {
    let control_block = DmaControlBlock::copy_slice(src, dst);
    
    DMA_0.start_transfer(&control_block);
    DMA_0.wait_for_end();
}

pub fn dma_copy_slice2d<T>(src: &Slice2d<T>, dst: &mut MutSlice2d<T>) {   
    let control_block = DmaControlBlock::copy_slice2d(src, dst);

    DMA_0.start_transfer(&control_block);
    DMA_0.wait_for_end();
}