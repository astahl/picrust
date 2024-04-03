use mystd::bit_field;


use super::mmio::Mmio;



pub struct SystemTimer{}

const STIMER_BASE: usize = 0x3000;

impl SystemTimer {
    const CS: Mmio<STIMER_BASE, 0x0> = Mmio();
    const CLO: Mmio<STIMER_BASE, 0x4> = Mmio();
    const CHI: Mmio<STIMER_BASE, 0x8> = Mmio();
    const C0: Mmio<STIMER_BASE, 0xc> = Mmio();
    const C1: Mmio<STIMER_BASE, 0x10> = Mmio();
    const C2: Mmio<STIMER_BASE, 0x14> = Mmio();
    const C3: Mmio<STIMER_BASE, 0x18> = Mmio();

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