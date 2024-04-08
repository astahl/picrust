use crate::bit_field;

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

#[repr(C, packed)]
pub struct DateOfManufacture {
    week_or_model_year_flag: u8,
    year: u8
}

pub union VideoInputParameters {
    digital: DigitalVideoInputParameters,
    analog: AnalogVideoInputParameters,
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
pub struct MonitorDescriptor {
    _reserved_zero_0: u16,
    _reserved_zero_1: u8,
    descriptor_type: u8,
    
}