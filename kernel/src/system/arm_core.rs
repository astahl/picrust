pub mod counter;
pub mod features;
pub mod mmu;
pub mod registers;

use registers::aarch64::general_sys_ctrl;
use registers::aarch64::special_purpose;

pub fn get_core_num() -> usize {
    general_sys_ctrl::mpidr_el1::read().cpu_id().value()
}

pub fn current_exception_level() -> usize {
    special_purpose::current_el().el().value()
}
