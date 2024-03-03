use mystd::bitfield::BitField;

use super::mmio::MMIO;



pub struct DmaChannel<const CHANNEL_BASE: usize>();

pub type Dma0 = DmaChannel<0x7000>;
pub type Dma1 = DmaChannel<0x7100>;
pub type Dma2 = DmaChannel<0x7200>;
pub type Dma3 = DmaChannel<0x7300>;
pub type Dma4 = DmaChannel<0x7400>;
pub type Dma5 = DmaChannel<0x7500>;
pub type Dma6 = DmaChannel<0x7600>;

impl<const CHANNEL_BASE: usize> DmaChannel<CHANNEL_BASE> {
    const CONTROL_STATUS: MMIO<CHANNEL_BASE, 0x00> = MMIO();
    const CONTROL_BLOCK_ADDRESS: MMIO<CHANNEL_BASE, 0x04> = MMIO();
    const TRANSFER_INFORMATION: MMIO<CHANNEL_BASE, 0x08> = MMIO();
    const SOURCE_ADDRESS: MMIO<CHANNEL_BASE, 0x0c> = MMIO();
    const DESTINATION_ADDRESS: MMIO<CHANNEL_BASE, 0x10> = MMIO();
    const TRANSFER_LENGTH: MMIO<CHANNEL_BASE, 0x14> = MMIO();
    const MODE_2D_STRIDE: MMIO<CHANNEL_BASE, 0x18> = MMIO();
    const NEXT_CONTROL_BLOCK_ADDRESS: MMIO<CHANNEL_BASE, 0x1c> = MMIO();
    const DEBUG: MMIO<CHANNEL_BASE, 0x20> = MMIO();

    pub fn control_status() -> DmaControlAndStatus {
        DmaControlAndStatus(BitField(Self::CONTROL_STATUS.read()))
    }

    pub fn set_control_status(cs: DmaControlAndStatus) {
        Self::CONTROL_STATUS.write(cs.value());
    }

    pub fn set_control_block_address(address: u32) {
        assert_eq!(0, address % core::mem::align_of::<DmaControlBlock>() as u32, "control block address must be aligned");
        Self::CONTROL_BLOCK_ADDRESS.write(address);
    }

    pub fn control_block_address() -> u32 {
        Self::CONTROL_BLOCK_ADDRESS.read()
    }
}

pub struct DmaControlAndStatus(BitField<u32>);

impl DmaControlAndStatus{
    pub fn new(value: u32) -> Self {
        Self(BitField(value))
    }

    pub fn value(&self) -> u32 {
        self.0.0
    }

    pub fn is_active(&self) -> bool {
        self.0.bit_test(0)
    }

    pub fn start(&mut self) {
        self.0.bit_set(0);
    }

    pub fn pause(&mut self) {
        self.0.bit_clear(0);
    }

    pub fn is_end(&self) -> bool {
        self.0.bit_test(1)
    }

    pub fn clear_end(&mut self) {
        self.0.bit_set(1);
    }

    pub fn is_interrupted(&self) -> bool {
        self.0.bit_test(2)
    }

    pub fn clear_interrupted(&mut self) {
        self.0.bit_set(2);
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

    pub fn set_axi_priority_level(&mut self, level: u32) {
        self.0.field_set(16, 4, level);
    }

    pub fn axi_panic_priority_level(&self) -> u32 {
        self.0.field(20, 4)
    }

    pub fn set_axi_panic_priority_level(&mut self, level: u32) {
        self.0.field_set(20, 4, level);
    }

    pub fn will_wait_for_outstanding_writes(&self) -> bool {
        self.0.bit_test(28)
    }

    pub fn set_will_wait_for_outstanding_writes(&mut self, should_wait: bool) {
        if should_wait { self.0.bit_set(28) } else { self.0.bit_clear(28) }
    }

    pub fn disable_debug_pause_signal(&mut self) {
        self.0.bit_set(29);
    }

    pub fn enable_debug_pause_signal(&mut self) {
        self.0.bit_clear(29);
    }

    pub fn is_debug_pause_signal_disabled(&self) -> bool {
        self.0.bit_test(29)
    }

    pub fn abort(&mut self) {
        self.0.bit_set(30);
    }

    pub fn is_aborting(&self) -> bool {
        self.0.bit_test(30)
    }

    pub fn reset(&mut self) {
        self.0.bit_set(31);
    }

    pub fn is_resetting(&self) -> bool {
        self.0.bit_test(31)
    }
}


#[repr(align(32))]
pub struct DmaControlBlock {
    transfer_information: DmaTransferInformation,
    source_address: u32,
    destination_address: u32,
    transfer_length: DmaTransferLength,
    stride: Dma2dStride,
    next_control_block_address: u32,
    reserved: [u32;2]
}


pub enum DmaTransferWidth {
    Bit32,
    Bit128
}

#[repr(transparent)]
pub struct DmaTransferInformation(BitField<u32>);

impl DmaTransferInformation {
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
    pub fn new(source: i16, destination: i16) -> Self {
        Self{ destination, source }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Clone, Copy)]
struct DmaTransferLength2d {
    x_byte_len: u16, y_count: u16
}

pub union DmaTransferLength {
    linear: u32,
    two_d: DmaTransferLength2d
}

impl DmaTransferLength {
    pub fn new_2d(x_byte_len: u16, y_count: u16) -> Self {
        assert!(y_count < 0x4000);
        Self { two_d: DmaTransferLength2d { x_byte_len, y_count } }
    }

    pub fn new_linear(byte_len: u32) -> Self {
        assert!(byte_len < 0x4000_0000);
        Self{ linear: byte_len }
    }

    pub fn linear(self) -> u32 {
        unsafe { self.linear & 0x3FFF_FFFF }
    }

    pub fn x_byte_len(self) -> u16 {
        unsafe { self.two_d.x_byte_len }
    }

    pub fn y_count(self) -> u16 {
        unsafe { self.two_d.y_count & 0x3FFF }
    }
}

