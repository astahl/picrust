


const USB_BASE: usize = 0x980000;
const USB_CORE_BASE: usize = USB_BASE;
const USB_HOST_BASE: usize = USB_BASE + 0x400;
const USB_POWER_BASE: usize = USB_BASE + 0xe00;


use super::mmio::TypedMMIO;
use mystd::bitfield::BitField;

type DwhciCoreAhbCfgReg = TypedMMIO<DwHciCoreAhbCfg, USB_CORE_BASE, 0x008>;
type DwhciCoreUsbCfgReg = TypedMMIO<DwHciCoreUsbCfg, USB_CORE_BASE, 0x00c>;
type DwhciCoreResetReg = TypedMMIO<DwHciCoreReset, USB_CORE_BASE, 0x010>;
type DwhciCoreIntStatReg = TypedMMIO<DwHciCoreInterrupts, USB_CORE_BASE, 0x014>;
type DwhciCoreIntMaskReg = TypedMMIO<DwHciCoreInterrupts, USB_CORE_BASE, 0x018>;
type DwhciCoreUserIdReg = TypedMMIO<u32, USB_CORE_BASE, 0x03c>;
type DwhciCoreVendorIdReg = TypedMMIO<u32, USB_CORE_BASE, 0x040>;
type DwhciCoreHwCfg1Reg = TypedMMIO<DwHciCoreHwCfg1, USB_CORE_BASE, 0x044>;
type DwhciCoreHwCfg2Reg = TypedMMIO<DwHciCoreHwCfg2, USB_CORE_BASE, 0x048>;
type DwhciCoreHwCfg3Reg = TypedMMIO<DwHciCoreHwCfg3, USB_CORE_BASE, 0x04c>;
type DwhciCoreHwCfg4Reg = TypedMMIO<DwHciCoreHwCfg4, USB_CORE_BASE, 0x050>;


#[derive(Clone, Copy)]
pub struct DwHciCore {}

impl DwHciCore {
    pub fn vendor_id() -> u32 {
        DwhciCoreVendorIdReg::read()
    }

    pub fn ahb_config() -> DwHciCoreAhbCfg {
        DwhciCoreAhbCfgReg::read()
    }

    pub fn set_ahb_config(config: DwHciCoreAhbCfg) {
        DwhciCoreAhbCfgReg::write(config)
    }

    pub fn usb_config() -> DwHciCoreUsbCfg {
        DwhciCoreUsbCfgReg::read()
    }

    pub fn set_usb_config(config: DwHciCoreUsbCfg) {
        DwhciCoreUsbCfgReg::write(config)
    }

    pub fn get_reset() -> DwHciCoreReset {
        DwhciCoreResetReg::read()
    }

    pub fn set_reset(reset: DwHciCoreReset) {
        DwhciCoreResetReg::write(reset)
    }

    pub fn hw_config() -> (DwHciCoreHwCfg1, DwHciCoreHwCfg2, DwHciCoreHwCfg3, DwHciCoreHwCfg4) {
        (
            DwhciCoreHwCfg1Reg::read(),
            DwhciCoreHwCfg2Reg::read(),
            DwhciCoreHwCfg3Reg::read(),
            DwhciCoreHwCfg4Reg::read(),
        )
    }
    
    pub fn interrupt_state() -> DwHciCoreInterrupts {
        DwhciCoreIntStatReg::read()
    }

    pub fn set_interrupt_state(state: DwHciCoreInterrupts) {
        DwhciCoreIntStatReg::write(state)
    }
}

#[derive(Clone, Copy)]
pub struct DwHciCoreAhbCfg(BitField<u32>);

impl DwHciCoreAhbCfg {
    pub fn is_global_interrupt_enabled(self) -> bool {
        self.0.bit_test(0)
    }

    pub fn with_global_interrupt_enabled(self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    pub fn with_global_interrupt_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }

    pub fn is_dma_enabled(self) -> bool {
        self.0.bit_test(5)
    }

    pub fn with_dma_enabled(self) -> Self {
        Self(self.0.with_bit_set(5))
    }

    pub fn with_dma_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(5))
    }

    pub fn is_wait_axi_writes_set(self) -> bool {
        self.0.bit_test(4)
    }

    pub fn with_wait_axi_writes_set(self) -> Self {
        Self(self.0.with_bit_set(4))
    }

    pub fn with_wait_axi_writes_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(4))
    }

    pub fn with_max_axi_burst(self, value: u32) -> Self {
        Self(self.0.with_field_set(1, 2, value))
    }

    pub fn max_axi_burst(self) -> u32 {
        self.0.field(1, 2)
    }
}


#[derive(Clone, Copy)]
pub struct DwHciCoreUsbCfg(BitField<u32>);

impl DwHciCoreUsbCfg {
	const PHYIF: usize =		3;
	const ULPI_UTMI_SEL: usize =	4;
	const SRP_CAPABLE: usize = 		8;
	const HNP_CAPABLE: usize = 		9;
	const ULPI_FSLS: usize =		17;
	const ULPI_CLK_SUS_M: usize =	19;
	const ULPI_EXT_VBUS_DRV: usize =	20;
	const TERM_SEL_DL_PULSE: usize =	22;

    pub fn is_phyif_set(self) -> bool {
        self.0.bit_test(Self::PHYIF)
    }

    pub fn with_phyif_set(self) -> Self {
        Self(self.0.with_bit_set(Self::PHYIF))
    }

    pub fn with_phyif_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::PHYIF))
    }

    pub fn is_ulpi_utmi_sel_set(self) -> bool {
        self.0.bit_test(Self::ULPI_UTMI_SEL)
    }

    pub fn with_ulpi_utmi_sel_set(self) -> Self {
        Self(self.0.with_bit_set(Self::ULPI_UTMI_SEL))
    }

    pub fn with_ulpi_utmi_sel_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::ULPI_UTMI_SEL))
    }

    pub fn is_srp_capable_set(self) -> bool {
        self.0.bit_test(Self::SRP_CAPABLE)
    }

    pub fn with_srp_capable_set(self) -> Self {
        Self(self.0.with_bit_set(Self::SRP_CAPABLE))
    }

    pub fn with_srp_capable_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::SRP_CAPABLE))
    }

    pub fn is_hnp_capable_set(self) -> bool {
        self.0.bit_test(Self::HNP_CAPABLE)
    }

    pub fn with_hnp_capable_set(self) -> Self {
        Self(self.0.with_bit_set(Self::HNP_CAPABLE))
    }

    pub fn with_hnp_capable_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::HNP_CAPABLE))
    }

    pub fn is_ulpi_fsls_set(self) -> bool {
        self.0.bit_test(Self::ULPI_FSLS)
    }

    pub fn with_ulpi_fsls_set(self) -> Self {
        Self(self.0.with_bit_set(Self::ULPI_FSLS))
    }

    pub fn with_ulpi_fsls_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::ULPI_FSLS))
    }

    pub fn is_ulpi_clk_sus_m_set(self) -> bool {
        self.0.bit_test(Self::ULPI_CLK_SUS_M)
    }

    pub fn with_ulpi_clk_sus_m_set(self) -> Self {
        Self(self.0.with_bit_set(Self::ULPI_CLK_SUS_M))
    }

    pub fn with_ulpi_clk_sus_m_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::ULPI_CLK_SUS_M))
    }

    pub fn is_ulpi_ext_vbus_drv_set(self) -> bool {
        self.0.bit_test(Self::ULPI_EXT_VBUS_DRV)
    }

    pub fn with_ulpi_ext_vbus_drv_set(self) -> Self {
        Self(self.0.with_bit_set(Self::ULPI_EXT_VBUS_DRV))
    }

    pub fn with_ulpi_ext_vbus_drv_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::ULPI_EXT_VBUS_DRV))
    }

    pub fn is_term_sel_dl_pulse_set(self) -> bool {
        self.0.bit_test(Self::TERM_SEL_DL_PULSE)
    }

    pub fn with_term_sel_dl_pulse_set(self) -> Self {
        Self(self.0.with_bit_set(Self::TERM_SEL_DL_PULSE))
    }

    pub fn with_term_sel_dl_pulse_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::TERM_SEL_DL_PULSE))
    }
}

#[derive(Clone, Copy)]
pub struct DwHciCoreReset(BitField<u32>);

impl DwHciCoreReset {
    const SOFT_RESET: usize = 0;
    const RX_FIFO_FLUSH: usize = 4;
    const TX_FIFO_FLUSH: usize = 5;
    const TX_FIFO_NUM_FIELD_LSB: usize = 6;
    const TX_FIFO_NUM_FIELD_WIDTH: usize = 5;
    const AHB_IDLE: usize = 31;
    
    pub fn clear() -> Self {
        Self(BitField::zero())
    }

    pub fn is_soft_reset_set(self) -> bool {
        self.0.bit_test(Self::SOFT_RESET)
    }

    pub fn with_soft_reset_set(self) -> Self {
        Self(self.0.with_bit_set(Self::SOFT_RESET))
    }

    pub fn with_soft_reset_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::SOFT_RESET))
    }

    pub fn is_rx_fifo_flush_set(self) -> bool {
        self.0.bit_test(Self::RX_FIFO_FLUSH)
    }

    pub fn with_rx_fifo_flush_set(self) -> Self {
        Self(self.0.with_bit_set(Self::RX_FIFO_FLUSH))
    }

    pub fn with_rx_fifo_flush_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::RX_FIFO_FLUSH))
    }

    pub fn is_tx_fifo_flush_set(self) -> bool {
        self.0.bit_test(Self::TX_FIFO_FLUSH)
    }

    pub fn with_tx_fifo_flush_set(self) -> Self {
        Self(self.0.with_bit_set(Self::TX_FIFO_FLUSH))
    }

    pub fn with_tx_fifo_flush_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::TX_FIFO_FLUSH))
    }

    pub fn tx_fifo_num(self) -> u32 {
        self.0.field(Self::TX_FIFO_NUM_FIELD_LSB, Self::TX_FIFO_NUM_FIELD_WIDTH)
    }

    pub fn with_tx_fifo_num(self, value: u32) -> Self {
        Self(self.0.with_field_set(Self::TX_FIFO_NUM_FIELD_LSB, Self::TX_FIFO_NUM_FIELD_WIDTH, value))
    }

    pub fn is_ahb_idle_set(self) -> bool {
        self.0.bit_test(Self::AHB_IDLE)
    }

    pub fn with_ahb_idle_set(self) -> Self {
        Self(self.0.with_bit_set(Self::AHB_IDLE))
    }

    pub fn with_ahb_idle_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(Self::AHB_IDLE))
    }
}



#[derive(Clone, Copy)]
pub struct DwHciCoreHwCfg1(BitField<u32>);
#[derive(Clone, Copy)]
pub struct DwHciCoreHwCfg2(BitField<u32>);
#[derive(Clone, Copy)]
pub struct DwHciCoreHwCfg3(BitField<u32>);
#[derive(Clone, Copy)]
pub struct DwHciCoreHwCfg4(BitField<u32>);

#[repr(u32)]
pub enum HsPhyType {
    NotSupported = 0b00,
    Utmi = 0b01,
    Ulpi = 0b10,
    UtmiAndUlpi = 0b11
}

#[repr(u32)]
pub enum FsPhyType {
    Unknown0 = 0b00,
    Dedicated = 0b01,
    Unknown2 = 0b10,
    Unknown3 = 0b11
}

impl DwHciCoreHwCfg2 {
    pub fn op_mode(self) -> u32 {
        self.0.field(0, 3)
    }

    pub fn architecture(self) -> u32 {
        self.0.field(3, 2)
    }

    pub fn hs_phy_type(self) -> HsPhyType {
        unsafe { core::mem::transmute(self.0.field(6, 2)) }
    }

    pub fn fs_phy_type(self) -> FsPhyType {
        unsafe { core::mem::transmute(self.0.field(8, 2)) }
    }

    pub fn num_host_channels(self) -> u32 {
        self.0.field(14, 4) + 1
    }
}

impl DwHciCoreHwCfg3 {
    pub fn dfifo_depth(self) -> u32 {
        self.0.field(16, 16)
    }
}

impl DwHciCoreHwCfg4 {
    pub fn is_ded_fifo_enabled(self) -> bool {
        self.0.bit_test(25)
    }

    pub fn num_in_eps(self) -> u32 {
        self.0.field(26, 4)
    }
}


#[derive(Clone, Copy)]
pub struct DwHciCoreInterrupts (BitField<u32>);

impl DwHciCoreInterrupts {
    pub const MODE_MISMATCH: usize = 1;
    pub const SOF: usize = 3;
    pub const RX_STS_Q_LVL: usize = 4;
    pub const USB_SUSPEND: usize = 11;
    pub const PORT: usize = 24;
    pub const HC: usize = 25;
    pub const CON_ID_STS_CHNG: usize = 28;
    pub const DISCONNECT: usize = 29;
    pub const SESS_REQ: usize = 30;
    pub const WKUP: usize = 31;
    
    pub fn clear() -> Self {
        Self(BitField::zero())
    }

    pub fn all_set() -> Self {
        Self(BitField::new(u32::MAX))
    }

    pub fn is_set(self, interrupt: usize) -> bool {
        self.0.bit_test(interrupt)
    }

    pub fn with_set(self, interrupt: usize) -> Self {
        Self(self.0.with_bit_set(interrupt))
    }

    pub fn with_cleared(self, interrupt: usize) -> Self {
        Self(self.0.with_bit_cleared(interrupt))
    }
}
