use core::arch::asm;

use crate::system::arm_core::registers::aarch64::special_purpose;

pub struct TpidrElx {}


impl TpidrElx {
    pub fn read_register() -> u64 {
        match special_purpose::current_el().el().value() {
            0 => Self::read_register_el0(),
            1 => Self::read_register_el1(),
            2 => Self::read_register_el2(),
            3 => Self::read_register_el3(),
            _ => unreachable!()
        }
    }

    pub fn write_register(thread_id: u64) {
        match special_purpose::current_el().el().value() {
            0 => Self::write_register_el0(thread_id),
            1 => Self::write_register_el1(thread_id),
            2 => Self::write_register_el2(thread_id),
            3 => Self::write_register_el3(thread_id),
            _ => unreachable!()
        };
    }

    pub fn read_register_el0() -> u64 {
        let value: u64;
        unsafe { asm!("mrs {0}, tpidr_el0", out(reg) value) };
        value
    }

    pub fn read_register_el1() -> u64 {
        let value: u64;
        unsafe { asm!("mrs {0}, tpidr_el1", out(reg) value) };
        value
    }

    pub fn read_register_el2() -> u64 {
        let value: u64;
        unsafe { asm!("mrs {0}, tpidr_el2", out(reg) value) };
        value
    }

    pub fn read_register_el3() -> u64 {
        let value: u64;
        unsafe { asm!("mrs {0}, tpidr_el3", out(reg) value) };
        value
    }

    pub fn write_register_el0(thread_id: u64) {
        unsafe { asm!("msr tpidr_el0, {}", in(reg) thread_id) };
    }

    pub fn write_register_el1(thread_id: u64) {
        unsafe { asm!("msr tpidr_el1, {}", in(reg) thread_id) };
    }

    pub fn write_register_el2(thread_id: u64) {
        unsafe { asm!("msr tpidr_el2, {}", in(reg) thread_id) };
    }

    pub fn write_register_el3(thread_id: u64) {
        unsafe { asm!("msr tpidr_el3, {}", in(reg) thread_id) };
    }
}