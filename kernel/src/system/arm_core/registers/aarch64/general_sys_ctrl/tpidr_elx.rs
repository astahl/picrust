use core::arch::asm;

use special_purpose::ExceptionLevel;

use crate::system::arm_core::{self, registers::aarch64::special_purpose};

pub struct TpidrElx {}


impl TpidrElx {
    pub fn read_register() -> u64 {
        match arm_core::current_exception_level() {
            ExceptionLevel::EL0 => Self::read_register_el0(),
            ExceptionLevel::EL1 => Self::read_register_el1(),
            ExceptionLevel::EL2 => Self::read_register_el2(),
            ExceptionLevel::EL3 => Self::read_register_el3(),
            //_ => unreachable!()
        }
    }

    pub fn write_register(thread_id: u64) {
        match arm_core::current_exception_level() {
            ExceptionLevel::EL0 => Self::write_register_el0(thread_id),
            ExceptionLevel::EL1 => Self::write_register_el1(thread_id),
            ExceptionLevel::EL2 => Self::write_register_el2(thread_id),
            ExceptionLevel::EL3 => Self::write_register_el3(thread_id),
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