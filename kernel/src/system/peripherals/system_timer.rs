use mystd::bit_field;

use crate::println_debug;

use super::mmio::MMIO;



pub struct SystemTimer{}

const STIMER_BASE: usize = 0x3000;

impl SystemTimer {
    const CS: MMIO<STIMER_BASE, 0x0> = MMIO();
    const CLO: MMIO<STIMER_BASE, 0x4> = MMIO();
    const CHI: MMIO<STIMER_BASE, 0x8> = MMIO();
    const C0: MMIO<STIMER_BASE, 0xc> = MMIO();
    const C1: MMIO<STIMER_BASE, 0x10> = MMIO();
    const C2: MMIO<STIMER_BASE, 0x14> = MMIO();
    const C3: MMIO<STIMER_BASE, 0x18> = MMIO();

    pub fn counter_low() -> u32 {
        Self::CLO.read()
    }

    pub fn counter_high() -> u32 {
        Self::CHI.read()
    }

    pub fn counter() -> u64 {
        ((Self::CHI.read() as u64) << 32) | Self::CLO.read() as u64
    }

    pub fn clear_matches(cs: ControlAndStatus) {
        Self::CS.write(0b1111 & cs.0);
    }

    pub fn matches() -> ControlAndStatus {
        ControlAndStatus(Self::CS.read() & 0b1111)
    }

    pub fn set_compare_0(value: u32) {
        Self::C0.write(value)
    }

    pub fn set_compare_1(value: u32) {
        Self::C1.write(value)
    }

    pub fn set_compare_2(value: u32) {
        Self::C2.write(value)
    }

    pub fn set_compare_3(value: u32) {
        Self::C3.write(value)
    }

    pub fn compare_0() -> u32 {
        Self::C0.read()
    }

    pub fn compare_1() -> u32 {
        Self::C1.read()
    }

    pub fn compare_2() -> u32 {
        Self::C2.read()
    }

    pub fn compare_3() -> u32 {
        Self::C3.read()
    }
}

bit_field!(pub ControlAndStatus(u32) {
    3 => match_3,
    2 => match_2,
    1 => match_1,
    0 => match_0,
});