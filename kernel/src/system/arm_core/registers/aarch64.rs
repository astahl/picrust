pub mod general_sys_ctrl;
pub mod special_purpose;
pub mod generic_timer;

#[macro_export]
/// Use this macro to create a system register access pattern impl for a bit_field type.
/// 
/// `system_register_impl!(<register_id> <type_id> (<access-option>,*))`
/// 
/// ## access-option: 
/// 
/// * `r` creates a `read_register() -> Self` accessor function.
/// * `w` creates a `write_register(self) -> ()` accessor function.
/// * `read_ordered` creates a `read_register_ordered() -> Self` accessor function, that uses a data barrier to ensure all loads and stores have happened before reading the register.
/// * `read_ordered_ish` creates a `read_register_ordered_ish() -> Self` accessor function, that uses a data barrier with the ish option.
/// 
/// # Usage 
/// 
/// ```rust
/// system_register_impl!(cpuectlr_el1 CpuECtlREl1 (r,w));
/// 
/// bit_field!(pub CpuECtlREl1 (u64) {
///     3 => i,
///     2 => enabled
/// });
/// ```
macro_rules! system_register_impl {

    ($reg_id:ident $type_name:ident) => {};
    ($reg_id:ident $type_name:ident r$(,$opts:tt)*) => {
        /// Reads the value from the associated system register
        #[inline]
        pub fn read_register() -> Self {
            let value: u64;
            unsafe { core::arch::asm!(concat!("mrs {}, ", stringify!($reg_id)), out(reg) value, options(nomem, nostack)) };
            Self::new(value)
        }

        $crate::system_register_impl!($reg_id $type_name $($opts),*);
    };
    ($reg_id:ident $type_name:ident w$(,$opts:tt)*) => {
        /// Writes the value to the associated system register
       #[inline]
        pub fn write_register(self) {
            unsafe { core::arch::asm!(concat!("msr ", stringify!($reg_id), ", {}"), in(reg) self.0, options(nomem, nostack)) };
        }
        $crate::system_register_impl!($reg_id $type_name $($opts),*);
    };
    ($reg_id:ident $type_name:ident read_ordered$(,$opts:tt)*) => {
        /// Reads the value from the associated system register, ordering access using memory barrier instructions `dsb` and `isb`. 
        #[inline]
        pub fn read_register_ordered() -> Self {
            let value: u64;
            unsafe { core::arch::asm!(
                "dsb",
                "isb",
                concat!("mrs {}, ", stringify!($reg_id)), out(reg) value, options(nostack)) };
            Self::new(value)
        }
        $crate::system_register_impl!($reg_id $type_name $($opts),*);
    };
    ($reg_id:ident $type_name:ident read_ordered_ish$(,$opts:tt)*) => {
        /// Reads the value from the associated system register, ordering access using memory barrier instructions `dsb ish` and `isb`. 
        #[inline]
        pub fn read_register_ordered_ish() -> Self {
            let value: u64;
            unsafe { core::arch::asm!(
                "dsb ish",
                "isb",
                concat!("mrs {}, ", stringify!($reg_id)), out(reg) value, options(nostack)) };
            Self::new(value)
        }
        $crate::system_register_impl!($reg_id $type_name $($opts),*);
    };
    ($reg_id:ident $type_name:ident ($($opts:tt),+)) => {
        impl $type_name {
            $crate::system_register_impl!($reg_id $type_name $($opts),+);
        }
    };

}