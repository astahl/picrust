use mystd::bit_field;

use crate::system_register_impl;

system_register_impl!(hcr_el2 HcrEl2 (r,w));

bit_field!(
    /// # HCR_EL2, Hypervisor Configuration Register
    /// Provides configuration controls for virtualization, including defining whether various operations are trapped to EL2.
    pub HcrEl2(u64) {
    
    #[cfg(feature = "feat_twed")]
    63:60 => twedel,
    #[cfg(feature = "feat_twed")]
    59 => twed_en,
    58 => tid5,
    57 => dct,
    56 => ata,
    55 => ttlbos,
    54 => ttlbis,
    53 => en_scxt,
    52 => tocu,
    51 => amvoffen,
    50 => ticab,
    49 => tid4,
    48 => gpf,
    47 => fien,
    46 => fwb,
    45 => nv2,
    44 => at,
    43 => nv1,
    42 => nv,
    41 => api,
    40 => apk,
    39 => tme,
    38 => miocnce,
    37 => tea,
    36 => terr,
    35 => tlor,
    34 => e2h,
    33 => id,
    32 => cd,
    31 => rw,
    30 => trvm,
    29 => hcd,
    28 => tdz,
    27 => tge,
    26 => tvm,
    25 => ttlb,
    24 => tpu,
    23 => tpcp,
    22 => tsw,
    21 => tacr,
    20 => tidcp,
    19 => tsc,
    18 => tid3,
    17 => tid2,
    16 => tid1,
    15 => tid0,
    14 => twe,
    13 => twi,
    12 => dc,
    11:10 => bsu,
    9 => fb,
    8 => vse,
    7 => vi,
    6 => vf,
    5 => amo,
    4 => imo,
    3 => fmo,
    2 => ptw,
    1 => swio = true,
    0 => vm,
});