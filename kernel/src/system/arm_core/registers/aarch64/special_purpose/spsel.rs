use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(spsel SpSel (r,w));

bit_field!(pub SpSel(u64){
    /// # SP, bit \[0]
    /// Stack pointer to use.
    /// The reset behavior of this field is:
    /// * On a Warm reset, this field resets to 1.
    0 => sp: enum SpToUse {
        /// Use SP_EL0 at all Exception levels.
        UseSpEl0,
        /// Use SP_ELx for Exception level ELx.
        /// When FEAT_NMI is implemented and SCTLR_ELx.SPINTMASK is 1, if execution is at ELx, an IRQ or FIQ interrupt that is targeted to ELx is masked regardless of any denotion of Superpriority.
        UseSpEln 
    }
});
