use crate::system::arm_core::registers::aarch64::general_sys_ctrl::tpidr_elx::TpidrElx;



pub fn id() -> u64 {
    TpidrElx::read_register()
}

pub fn set_id(thread_id: u64) {
    TpidrElx::write_register(thread_id)
}

pub fn spin_wait_cycles(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}


pub fn spin_wait_for(duration: core::time::Duration) {
    let now = super::counter::PointInTime::now();
    let expire_at = now + duration;
    loop {
        core::hint::spin_loop();
        if !expire_at.is_in_the_future() {
            break;
        }
    }
}

