


const USB_BASE: usize = 0x980000;
const USB_CORE_BASE: usize = USB_BASE;
const USB_HOST_BASE: usize = USB_BASE + 0x400;
const USB_POWER_BASE: usize = USB_BASE + 0xe00;


use super::mmio::TypedMMIO;
use mystd::bit_field;

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

bit_field!(pub DwHciCoreAhbCfg(u32)
    0 => enable_global_interrupt,
    1:2 => max_axi_burst,
    4 => wait_axi_writes,
    5 => enable_dma
);


bit_field!(pub DwHciCoreUsbCfg(u32)
    3 => phyif,
    4 => ulpi_utmi_sel,
    8 => srp_capable,
    9 => hnp_capable,
    17 => ulpi_fsls,
    19 => ulpi_clk_sus_m,
    20 => ulpi_ext_vbus_drv,
    22 => term_sel_dl_pulse
);


bit_field!(pub DwHciCoreReset(u32)
    0 => soft_reset,
    4 => rx_fifo_flush,
    5 => tx_fifo_flush,
    6:10 => tx_fifo_num,
    31 => ahb_idle
);


bit_field!(pub DwHciCoreHwCfg1(u32));
bit_field!(pub DwHciCoreHwCfg2(u32)
    0:2 => op_mode,
    3:4 => architecture,
    6:7 => hs_phy_type: HsPhyType,
    8:9 => fs_phy_type: FsPhyType,
    14:17 => num_host_channels
);
bit_field!(pub DwHciCoreHwCfg3(u32)
    16:31 => dfifo_depth
);

bit_field!(pub DwHciCoreHwCfg4(u32)
    25 => enable_ded_fifo,
    26:29 => num_in_eps
);

#[repr(u32)]
pub enum HsPhyType {
    NotSupported = 0b00,
    Utmi = 0b01,
    Ulpi = 0b10,
    UtmiAndUlpi = 0b11
}

impl From<u32> for HsPhyType {
    fn from(value: u32) -> Self {
        unsafe { core::mem::transmute(value & 0b11) }
    }
}

#[repr(u32)]
pub enum FsPhyType {
    Unknown0 = 0b00,
    Dedicated = 0b01,
    Unknown2 = 0b10,
    Unknown3 = 0b11
}

impl From<u32> for FsPhyType {
    fn from(value: u32) -> Self {
        unsafe { core::mem::transmute(value & 0b11) }
    }
}

impl DwHciCoreHwCfg2 {
    pub fn num_host_channels_actual(self) -> u32 {
        self.num_host_channels().value() + 1
    }
}


bit_field!(pub DwHciCoreInterrupts(u32)
    1 => mode_mismatch,
    3 => sof,
    4 => rx_sts_q_lvl,
    11 => usb_suspend,
    24 => port,
    25 => hc,
    28 => con_id_sts_chng,
    29 => disconnect,
    30 => sess_req,
    31 => wkup
);
