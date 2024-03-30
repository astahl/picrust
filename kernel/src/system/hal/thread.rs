use crate::system::arm_core::registers::aarch64::general_sys_ctrl::tpidr_elx::TpidrElx;



pub fn id() -> u64 {
    TpidrElx::read_register()
}

pub fn set_id(thread_id: u64) {
    TpidrElx::write_register(thread_id)
}