use crate::{bit_field, fixed_point::FixedPoint};

#[repr(C, packed)]
pub struct EdidVer14 {
    magic_number: u64,
    manufacturer_id: ManufacturerId,
    manufacturer_product_code: u16,
    serial_number: u32,
    date_of_manufacture: DateOfManufacture,
    edid_version: u8,
    edid_revision: u8,
    // basic display parameters
    video_input_parameters: VideoInputParameters,
    screen_size: ScreenSize,
    gamma: Gamma,
    supported_features: SupportedFeatures,
    
    // chromaticity
    chromaticity_coords: ChromaticityCoordinates,

    supported_common_timings: (EstablishedTimingModes0, EstablishedTimingModes1, EstablishedTimingModes2),
    standard_timings: [StandardTiming; 8],

    detailed_descriptors: [DetailedDescriptor; 4],
    num_of_extensions: u8,
    checksum_parity: u8,
}

bit_field!(pub ManufacturerId(u16) {
    14:10 => letter_0,
    9:5 => letter_1,
    4:0 => letter_2
} );

impl ManufacturerId {
    pub fn to_ascii(&self) -> [u8;3] {
        [
            self.letter_0().value() as u8 + b'@',
            self.letter_1().value() as u8 + b'@',
            self.letter_2().value() as u8 + b'@',
        ]
    }
}


#[repr(C, packed)]
pub struct DateOfManufacture {
    week_or_model_year_flag: u8,
    year_raw: u8
}

impl DateOfManufacture {
    pub fn year(&self) -> u16 {
        self.year_raw as u16 + 1990
    }
}

pub union VideoInputParameters {
    digital: DigitalVideoInputParameters,
    analog: AnalogVideoInputParameters,
}

impl VideoInputParameters {
    pub fn is_digital(&self) -> bool {
        unsafe { self.digital.is_digital().is_set() } 
    }

    pub fn is_analog(&self) -> bool {
        unsafe { self.digital.is_digital().is_clear() } 
    }

    pub fn try_as_digital(&self) -> Option<&DigitalVideoInputParameters> {
        unsafe {
            if self.digital.is_digital().is_set() {
                Some(&self.digital)
            } else {
                None
            }
        }
    }

    pub fn try_as_analog(&self) -> Option<&AnalogVideoInputParameters> {
        unsafe {
            if self.analog.is_digital().is_clear() {
                Some(&self.analog)
            } else {
                None
            }
        }
    }
}

bit_field!(pub DigitalVideoInputParameters (u8) {
    7 => is_digital,
    6:4 => bit_depth: enum BitDepth {
        Undefined = 0,
        BitsPerColor6 = 0b001,
        BitsPerColor8 = 0b010,
        BitsPerColor10 = 0b011,
        BitsPerColor12 = 0b100,
        BitsPerColor14 = 0b101,
        BitsPerColor16 = 0b110,
        Reserved = 0b111,
    },
    3:0 => video_interface: enum VideoInterface {
        Undefined = 0b0000,
        Dvi = 0b0001,
        HdmiA = 0b0010,
        HdmiB = 0b0011,
        Mddi = 0b0100,
        DisplayPort = 0b0101,
    },
});


bit_field!(pub AnalogVideoInputParameters (u8) {
    7 => is_digital,
    6:5 => video_white_and_sync_levels: enum VideoWhiteAndSyncLevels {
        Plus0_7Minus0_3 = 0b00,
        Plus0_714Minus0_286 = 0b01,
        Plus1Minus0_4 = 0b10,
        Plus0_7Minus0EVC = 0b11,
    },
    4 => blank_to_blank_setup_expected,
    3 => separate_sync_supported,
    2 => composite_sync_on_h_supported,
    1 => sync_on_green_supported,
    0 => vsync_pulse_serrated,
});

#[repr(C, packed)]
pub struct ScreenSize {
    horizontal: u8,
    vertical: u8
}

pub struct Gamma(u8);

bit_field!(pub SupportedFeatures (u8) {
    7 => dpms_standby_supported,
    6 => dpms_suspend_supported,
    5 => dpms_active_off_supported,
    4:3 => digital_display_type: enum DigitalDisplayType{
        Rgb444 = 0,
        Rgb444YCrCb444 = 0b01,
        Rgb444YCrCb422 = 0b10,
        Rgb444YCrCb444YCrCb422 = 0b11,
    },
    4:3 => analog_display_type: enum AnalogDisplayType {
        MonochromeOrGrayscale = 0b00,
        RgbColor = 0b01,
        NonRgbColor = 0b10,
    },
    2 => standard_srgb_color_space,
    1 => preferred_timing_mode,
    0 => continuous_timings
});


#[repr(C, packed)]
pub struct ChromaticityCoordinates {
    lsb: ChromaticityCoordinatesLsb,
    msb: ChromaticityCoordinatesMsb
}

impl ChromaticityCoordinates {
    pub fn red_x(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.red_x as u16) << 2 | self.lsb.red_x().value() as u16)
    }
    
    pub fn red_y(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.red_y as u16) << 2 | self.lsb.red_y().value() as u16)
    }

    pub fn green_x(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.green_x as u16) << 2 | self.lsb.green_x().value() as u16)
    }

    pub fn green_y(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.green_y as u16) << 2 | self.lsb.green_y().value() as u16)
    }

    pub fn blue_x(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.blue_x as u16) << 2 | self.lsb.blue_x().value() as u16)
    }

    pub fn blue_y(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.blue_y as u16) << 2 | self.lsb.blue_y().value() as u16)
    }

    pub fn white_x(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.white_x as u16) << 2 | self.lsb.white_x().value() as u16)
    }

    pub fn white_y(&self) -> FixedPoint<10, u16> {
        FixedPoint::new((self.msb.white_y as u16) << 2 | self.lsb.white_y().value() as u16)
    }
}



#[repr(C, packed)]
pub struct ChromaticityCoordinatesMsb {
    red_x: u8,
    red_y: u8,
    green_x: u8,
    green_y: u8,
    blue_x: u8,
    blue_y: u8,
    white_x: u8,
    white_y: u8,
}

bit_field!(ChromaticityCoordinatesLsb (u16) {
    15:14 => red_x,
    13:12 => red_y,
    11:10 => green_x,
    9:8 => green_y,
    7:6 => blue_x,
    5:4 => blue_y,
    3:2 => white_x,
    1:0 => white_y,
});


bit_field!(EstablishedTimingModes0 (u8) {
    7 => supports_720x400v70, 
    6 => supports_720x400v88, 
    5 => supports_640x480v60, 
    4 => supports_640x480v67, 
    3 => supports_640x480v72,
    2 => supports_640x480v75,
    1 => supports_800x600v56,
    0 => supports_800x600v60,
});

bit_field!(EstablishedTimingModes1 (u8) {
    7 => supports_800x600v72,
    6 => supports_800x600v75,
    5 => supports_832x624v75,
    4 => supports_1024x768v87_interlaced,
    3 => supports_1024x768v60,
    2 => supports_1024x768v70,
    1 => supports_1024x768v75,
    0 => supports_1280x1024v75,
});

bit_field!(EstablishedTimingModes2 (u8) {
    7 => supports_1152x870v75
});

bit_field!(StandardTiming (u16) {
    15:8 => x_resolution_raw,
    7:6 => aspect_ratio: enum AspectRatio {
        Ar16by10 = 0b00,
        Ar4by3 = 0b01,
        Ar5by4 = 0b10,
        Ar16by9 = 0b11
    },
    5:0 => vertical_frequency_raw,
});

impl StandardTiming {
    pub const fn is_used(&self) -> bool {
        self.0 != 0x0101
    }
}

pub union DetailedDescriptor {
    timing: TimingDescriptor,
    monitor: MonitorDescriptor,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TimingDescriptor {
    pixel_clock_raw: core::num::NonZeroU16,
    horizontal: FieldPixels,
    vertical: FieldPixels,
    blanking: BlankingPixels,
    image_size: ImageSize,
    horizontal_border_pixels: u8,
    vertical_border_lines: u8,
    features: TimingFeatures
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FieldPixels {
    active_lsb: u8,
    blanking_lsb: u8,
    msb: FieldPixelsMsb,
}

bit_field!(pub FieldPixelsMsb(u8) {
    7:4 => active_msb,
    3:0 => blanking_msb
});

impl FieldPixels {
    pub fn active(&self) -> u16 {
        ((self.msb.active_msb().value() as u16) << 8) | self.active_lsb as u16
    }

    pub fn blanking(&self) -> u16 {
        ((self.msb.blanking_msb().value() as u16) << 8) | self.blanking_lsb as u16
    }
}

bit_field!(BlankingPixels (u32){
    31:24 => horizontal_front_porch_lsb,
    23:16 => horizontal_sync_pulse_width_lsb,
    15:12 => vertical_front_porch_lsb,
    11:8 => vertical_sync_pulse_width_lsb,
    7:6 => horizontal_front_porch_msb,
    5:4 => horizontal_sync_pulse_width_msb,
    3:2 => vertical_front_porch_msb,
    1:0 => vertical_sync_pulse_width_msb,
});



impl BlankingPixels {
    pub fn horizontal_front_porch(&self) -> u16 {
        (self.horizontal_front_porch_msb().value() << 8) as u16 | self.horizontal_front_porch_lsb().value() as u16
    }

    pub fn horizontal_sync_pulse_width(&self) -> u16 {
        (self.horizontal_sync_pulse_width_msb().value() << 8) as u16 | self.horizontal_sync_pulse_width_lsb().value() as u16
    }

    pub fn vertical_front_porch(&self) -> u8 {
        (self.vertical_front_porch_msb().value() << 4) as u8 | self.vertical_front_porch_lsb().value() as u8
    }

    pub fn vertical_sync_pulse_width(&self) -> u8 {
        (self.vertical_sync_pulse_width_msb().value() << 4) as u8 | self.vertical_sync_pulse_width_lsb().value() as u8
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ImageSize {
    horizontal_lsb: u8,
    vertical_lsb: u8,
    msb: ImageSizeMsb,
}

bit_field!(pub ImageSizeMsb(u8) {
    7:4 => horizontal_msb,
    3:0 => vertical_msb
});

impl ImageSize {
    pub fn horizontal(&self) -> u16 {
        ((self.msb.horizontal_msb().value() as u16) << 8) | self.horizontal_lsb as u16
    }

    pub fn vertical(&self) -> u16 {
        ((self.msb.vertical_msb().value() as u16) << 8) | self.vertical_lsb as u16
    }
}



bit_field!(pub TimingFeatures(u8) {
    7 => signal_interface_type: enum SignalInterfaceType {
        NonInterlaced,
        Interlaced
    }, 
    6:5 => stereo_mode_msb,
    4:3 => sync_type: enum SyncType {
        AnalogComposite = 0b00,
        BipolarAnalogComposite = 0b01,
        DigitalComposite = 0b10,
        DigitalSeparate = 0b11,
    },
    2 => serration, 
    2 => digital_separate_vertical_polarity: enum Polarity {
        Negative,
        Positive,
    },
    1 => analog_sync_on: enum SyncOn {
        SyncOnGreen,
        SyncOnRgb,
    },
    1 => digital_horizontal_polarity: Polarity,
    0 => stereo_mode_lsb,
});

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub union MonitorDescriptor {
    _tag: ([u8;3], u8, [u8;14]),
    // FF
    serial_number: MonitorDescriptorText,
    // FE
    unspecified_text: MonitorDescriptorText,
    // FD
    range_limits: MonitorRangeLimits,
    // FC
    monitor_name: MonitorDescriptorText,
    // FB
   // additional_white_point: MonitorAdditionalWhitePoint,
    
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct MonitorDescriptorText {
    _res_zero_0: [u8;3],
    /// Must be FF, FE, or FC 
    tag: u8,
    _res_zero_1: [u8;1],
    text_cp437: [u8;13]
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct MonitorRangeLimits {
    _res_zero_0: [u8;3],
    /// Must be FD
    tag: u8,
    offset_flags: MonitorRangeLimitsOffsets,
    minimum_vertical_field_rate_hz: u8,
    maximum_vertical_field_rate_hz: u8,
    minimum_horizontal_line_rate_khz: u8,
    maximum_horizontal_line_rate_khz: u8,
    maximum_pixel_clock_rate_10mhz: u8,
    extended_timing_information: MonitorVideoTimingParameters
}

bit_field!(pub MonitorRangeLimitsOffsets(u8){
    7:4 => res_0,
    3:2 => horizontal_offsets: enum MinMaxOffsetFlags {
        None = 0b00,
        Max = 0b10,
        MaxAndMin = 0b11,
    },
    1:0 => vertical_offsets: MinMaxOffsetFlags
});

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub union MonitorVideoTimingParameters {
    _tag: (u8, [u8;7]),
    gtf: MonitorVideoTimingGtf,
}

impl MonitorVideoTimingParameters {
    const EMPTY_PADDING: [u8;7] = [0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
    pub fn is_use_default_gtf(&self) -> bool {
        debug_assert_eq!(Self::EMPTY_PADDING, unsafe {self._tag.1});
        unsafe { self._tag.0 == 0 }
    }

    pub fn is_no_timing_information(&self) -> bool {
        debug_assert_eq!(Self::EMPTY_PADDING, unsafe {self._tag.1});
        unsafe { self._tag.0 == 1 }
    }

    pub fn is_secondary_gtf(&self) -> bool {
        unsafe { self._tag.0 == 2 }
    }

    pub fn is_cvt(&self) -> bool {
        unsafe { self._tag.0 == 4 }
    }

    pub fn try_as_secondary_gtf(&self) -> Option<&MonitorVideoTimingGtf> {
        if self.is_secondary_gtf() {
            unsafe { Some(&self.gtf) }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MonitorVideoTimingGtf {
    tag: u8,
    res0: u8,
    start_frequency_khz: FixedPoint<-1, u8>,
    gtf_c: FixedPoint<1, u8>,
    gtf_m_raw: [u8;2],
    gtf_k: u8,
    gtf_j: FixedPoint<1, u8>,
}

impl MonitorVideoTimingGtf {
    pub const fn gtf_m(&self) -> u16 {
        u16::from_le_bytes(self.gtf_m_raw)
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MonitorVideoTimingCvt {
    tag: u8,
    version: CvtVersion,
    parameters: CvtParams,
    aspect_ratios: CvtAspectRatioFlags,
    preferred: CvtPreferences,
    scaling_support: CvtScalingSupport,
    preferred_vertical_refresh_rate: u8
}

bit_field!(pub CvtVersion (u8){
    7:4 => major,
    3:0 => minor,
});

bit_field!(pub CvtParams (u16){
    15:10 => additional_clock_precision_250khz,
    9:0 => maximum_active_pixels_per_line,
});

bit_field!(pub CvtAspectRatioFlags (u8){
    7 => aspect_ratio_4_3,
    6 => aspect_ratio_16_9,
    5 => aspect_ratio_16_10,
    4 => aspect_ratio_5_4,
    3 => aspect_ratio_15_9,
});

bit_field!(pub CvtPreferences (u8){
    7:5 => aspect_ratio: enum CvtAspectRatioPreference {
        Ar4by3 = 0b000,
        Ar16by9 = 0b001,
        Ar16by10 = 0b010,
        Ar5by4 = 0b011,
        Ar15by9 = 0b100,
    },
    4 => reduced_blanking,
    3 => standard_blanking,
});

bit_field!(pub CvtScalingSupport (u8){
    7 => horizontal_shrink,
    6 => horizontal_stretch,
    5 => vertical_shrink,
    4 => vertical_stretch,
});


