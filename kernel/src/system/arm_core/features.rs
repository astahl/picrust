pub const FEAT_ETMV4: bool = cfg!(feature = "arm_feat_etmv4");
pub const FEAT_LPA2: bool = cfg!(feature = "arm_feat_lpa2");
pub const FEAT_LPA: bool = cfg!(feature = "arm_feat_lpa");
pub const FEAT_HPDS2: bool = cfg!(feature = "arm_feat_hpds2");
pub const FEAT_PAUTH: bool = cfg!(feature = "arm_feat_pauth");
/// ARMv8.2 SVE - Scalable Vector Extension
pub const FEAT_SVE: bool = cfg!(feature = "arm_feat_sve");
/// ARMv9 TME - Transactional Memory Extension
pub const FEAT_TME: bool = cfg!(feature = "arm_feat_tme");
