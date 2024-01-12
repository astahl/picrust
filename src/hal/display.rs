use core::fmt::{Debug, Display};

use crate::peripherals::mailbox;

#[derive(Debug)]
pub enum Edid {
    Edid(EdidBlock),
    CtaExtensionRev3(CtaExtensionBlock),
    Unknown,
}

impl Edid {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, ..] => {
                Self::Edid(EdidBlock(bytes.try_into().unwrap()))
            }
            [0x02, 0x03, ..] => {
                Self::CtaExtensionRev3(CtaExtensionBlock(bytes.try_into().unwrap()))
            }
            _ => Self::Unknown,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            Edid::Edid(b) => b.0.as_slice(),
            Edid::CtaExtensionRev3(b) => b.0.as_slice(),
            Edid::Unknown => &[],
        }
    }

    pub fn checksum_ok(&self) -> bool {
        self.bytes().iter().copied().reduce(u8::wrapping_add) == Some(0)
    }
}

pub struct EdidBlock([u8; 128]);

#[repr(u8)]
#[derive(Debug)]
pub enum BitDepth {
    Undefined,
    BitsPerColor6,
    BitsPerColor8,
    BitsPerColor10,
    BitsPerColor12,
    BitsPerColor14,
    BitsPerColor16,
    Reserved,
}

#[repr(u8)]
#[derive(Debug)]
pub enum VideoInterface {
    Undefined,
    DVI,
    HDMIa,
    HDMIb,
    MDDI,
    DisplayPort,
    Unknown6,
    Unknown7,
    Unknown8,
    Unknown9,
    Unknown10,
    Unknown11,
    Unknown12,
    Unknown13,
    Unknown14,
    Unknown15,
}

#[repr(u8)]
#[derive(Debug)]
pub enum VideoWhiteAndSyncLevels {
    Plus0_7Minus0_3,
    Plus0_714Minus0_286,
    Plus1Minus0_4,
    Plus0_7Minus0EVC,
}

#[derive(Debug)]
pub enum VideoInputParameter {
    Digital {
        bit_depth: BitDepth,
        video_interface: VideoInterface,
    },
    Analog {
        white_and_sync_levels: VideoWhiteAndSyncLevels,
        blank_to_blank_setup_expected: bool,
        separate_sync_supported: bool,
        composite_sync_on_h_supported: bool,
        sync_on_green_supported: bool,
        vsync_pulse_serrated: bool,
    },
}

#[derive(Debug)]
pub enum ScreenGeometry {
    Undefined,
    Landscape(u8),
    Portrait(u8),
    SizeInCentimeters { h: u8, v: u8 },
}

#[derive(Debug)]
pub enum DisplayTypeAnalog {
    Monochrome = 0b00,
    RGBColor = 0b01,
    NonRGBColor = 0b10,
    Undefined = 0b11,
}

#[derive(Debug)]
pub enum DisplayTypeDigital {
    RGB444 = 0b00,
    RGB444_YCbCr444 = 0b01,
    RGB444_YCbCr422 = 0b10,
    RGB444_YCbCr444_YCbCr422 = 0b11,
}

#[derive(Debug)]
pub enum DisplayType {
    Analog(DisplayTypeAnalog),
    Digital(DisplayTypeDigital),
}

#[derive(Debug)]
pub struct SupportedFeatures {
    dpms_standby_supported: bool,
    dpms_suspend_supported: bool,
    dpms_active_off_supported: bool,
    display_type: DisplayType,
    s_rgb: bool,
    preferred_timing_mode: bool,
    continuous_timings: bool,
}

#[derive(Debug, Default)]
pub struct CIEPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Default)]
pub struct ChromaticityCoordinates {
    red: CIEPoint,
    green: CIEPoint,
    blue: CIEPoint,
    white: CIEPoint,
}

#[derive(Debug, Default)]
pub struct CommonLegacyTimingSupport {
    _720_400_70: bool,
    _720_400_88: bool,
    _640_480_60: bool,
    _640_480_67: bool,
    _640_480_72: bool,
    _640_480_75: bool,
    _800_600_56: bool,
    _800_600_60: bool,
    _800_600_72: bool,
    _800_600_75: bool,
    _832_624_75: bool,
    _1024_768_87_interlaced: bool,
    _1024_768_60: bool,
    _1024_768_70: bool,
    _1024_768_75: bool,
    _1280_1024_75: bool,
    _1152_870_75: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum StandardTimingImageAspectRatio {
    #[default]
    _16_10,
    _4_3,
    _5_4,
    _16_9,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StandardTimingInformation {
    x_resolution: u16,
    image_aspect_ratio: StandardTimingImageAspectRatio,
    vertical_frequency: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum SignalInterfaceType {
    NonInterlaced,
    Interlaced,
}

#[derive(Debug, Clone, Copy)]
pub enum StereoMode {
    None = 0b00_0,
    Nope = 0b00_1, // bit set but don't care
    FieldSequentialRightDuringStereoSync = 0b01_0,
    FieldSequentialLeftDuringStereoSync = 0b10_0,
    TwoWayInterleavedRightOnEvenLines = 0b01_1,
    TwoWayInterleavedLeftOnEvenLines = 0b10_1,
    FourWayInterleaved = 0b11_0,
    SideBySideInterleaved = 0b11_1,
}

#[derive(Debug, Clone, Copy)]
pub enum Polarity {
    Negative,
    Positive,
}

#[derive(Debug, Clone, Copy)]
pub enum Sync {
    AnalogComposite {
        bipolar: bool,
        serrations_hsync_during_vsync: bool,
        sync_on_red_and_blue_lines_additionally_to_green: bool,
    },
    DigitalComposite {
        serrations_hsync_during_vsync: bool,
        horizontal_sync_polarity: Polarity,
    },
    DigitalSeparate {
        vertical_sync_polarity: Polarity,
        horizontal_sync_polarity: Polarity,
    },
}

#[derive(Clone, Copy)]
pub struct DescriptorText([u8; 13]);

impl core::fmt::Debug for DescriptorText {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl From<&[u8]> for DescriptorText {
    fn from(value: &[u8]) -> Self {
        Self(value.try_into().unwrap())
    }
}

impl DescriptorText {
    fn to_str(&self) -> &str {
        // actually it's Codepage 437, but let's just assume it's ascii
        core::str::from_utf8(self.0.as_slice()).unwrap().trim_end()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AspectRatioPreference {
    ar_4_3 = 0b000,
    ar_16_9 = 0b001,
    ar_16_10 = 0b010,
    ar_5_4 = 0b011,
    ar_15_9 = 0b100,
}

#[derive(Debug, Clone, Copy)]
pub enum ExtendedTimingInformation {
    // GTF = GeneralizedTimingFormula
    DefaultGtf,
    NoTimingInformation,
    SecondaryGtfSupported {
        // 0 - 510 khz
        start_frequency_2khz: u8,
        // 0 - 127.5
        gtf_c_0_5: u8,
        gtf_m: u16,
        gtf_k: u8,
        gtf_j_0_5: u8,
    },
    Cvt {
        cvt_version_major: u8,
        cvt_version_minor: u8,
        maximum_pixel_clock_reduction_250khz: u8,
        maximum_active_pixels_per_line: u16,
        ar_4_3: bool,
        ar_16_9: bool,
        ar_16_10: bool,
        ar_5_4: bool,
        ar_15_9: bool,
        preferred_aspect_ratio: AspectRatioPreference,
        preferred_reduced_blanking: bool,
        standard_blanking: bool,
        scaling_support_horizontal_shrink: bool,
        scaling_support_horizontal_stretch: bool,
        scaling_support_vertical_shrink: bool,
        scaling_support_vertical_stretch: bool,
        preferred_vertical_refresh_rate: u8,
    },
    Unknown(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum Descriptor {
    Unknown(u8),
    DetailedTiming {
        pixel_clock_10khz: u16,
        horizontal_active_pixels: u16,
        horizontal_blanking_pixels: u16,
        vertical_active_lines: u16,
        vertical_blanking_lines: u16,
        horizontal_front_porch_pixels: u16,
        horizontal_sync_pulse_width_pixels: u16,
        vertical_front_porch_lines: u8,
        vertical_sync_pulse_width_lines: u8,
        horizontal_image_size_mm: u16,
        vertical_image_size_mm: u16,
        horizontal_border_pixels: u8,
        vertical_border_lines: u8,
        signal_interface_type: SignalInterfaceType,
        stereo_mode: StereoMode,
        sync: Sync,
    },
    DisplaySerialNumber(DescriptorText),
    DisplayName(DescriptorText),
    UnspecifiedText(DescriptorText),
    RangeLimits {
        vertical_field_rate_hz_min_max: (u16, u16),
        horizontal_line_rate_khz_min_max: (u16, u16),
        maximum_pixel_clock_10mhz: u8,
        extended_timing_information: ExtendedTimingInformation,
    },
}

impl Descriptor {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 18 {
            return None;
        }
        let byte0 = bytes[0];
        let byte1 = bytes[1];
        match (byte0, byte1) {
            (0, 0) => match bytes[3] {
                0x10 => None,
                0xFF => Some(Descriptor::DisplaySerialNumber(
                    bytes[5..=17].try_into().unwrap(),
                )),
                0xFE => Some(Descriptor::UnspecifiedText(
                    bytes[5..=17].try_into().unwrap(),
                )),
                0xFC => Some(Descriptor::DisplayName(bytes[5..=17].try_into().unwrap())),
                0xFD => Some({
                    let mut vertical_field_rate_hz_min_max = (bytes[5] as u16, bytes[6] as u16);
                    let mut horizontal_line_rate_khz_min_max =
                        (bytes[7] as u16, bytes[8] as u16);

                    vertical_field_rate_hz_min_max.1 += (bytes[4] & 1) as u16 * 255;
                    vertical_field_rate_hz_min_max.0 += (bytes[4] >> 1 & 1) as u16 * 255;
                    horizontal_line_rate_khz_min_max.1 += (bytes[4] >> 2 & 1) as u16 * 255;
                    horizontal_line_rate_khz_min_max.0 += (bytes[4] >> 3 & 1) as u16 * 255;

                    Descriptor::RangeLimits {
                        vertical_field_rate_hz_min_max,
                        horizontal_line_rate_khz_min_max,
                        maximum_pixel_clock_10mhz: bytes[9],
                        extended_timing_information: match bytes[10] {
                            0 => ExtendedTimingInformation::DefaultGtf,
                            1 => ExtendedTimingInformation::NoTimingInformation,
                            2 => ExtendedTimingInformation::SecondaryGtfSupported {
                                start_frequency_2khz: bytes[12],
                                gtf_c_0_5: bytes[13],
                                gtf_m: u16::from_le_bytes(bytes[14..=15].try_into().unwrap()),
                                gtf_k: bytes[16],
                                gtf_j_0_5: bytes[17],
                            },
                            4 => ExtendedTimingInformation::Cvt {
                                cvt_version_major: bytes[11] >> 4,
                                cvt_version_minor: bytes[11] & 0xf,
                                maximum_pixel_clock_reduction_250khz: bytes[12] >> 2,
                                maximum_active_pixels_per_line: (bytes[12] as u16 & 0b11) << 8
                                    | bytes[13] as u16,
                                ar_4_3: bytes[14] & 0x80 != 0,
                                ar_16_9: bytes[14] & 0x40 != 0,
                                ar_16_10: bytes[14] & 0x20 != 0,
                                ar_5_4: bytes[14] & 0x10 != 0,
                                ar_15_9: bytes[14] & 0x08 != 0,
                                preferred_aspect_ratio: unsafe {
                                    core::mem::transmute(bytes[15] >> 5)
                                },
                                preferred_reduced_blanking: bytes[15] & 0x10 != 0,
                                standard_blanking: bytes[15] & 0x08 != 0,
                                scaling_support_horizontal_shrink: bytes[16] & 0x80 != 0,
                                scaling_support_horizontal_stretch: bytes[16] & 0x40 != 0,
                                scaling_support_vertical_shrink: bytes[16] & 0x20 != 0,
                                scaling_support_vertical_stretch: bytes[16] & 0x10 != 0,
                                preferred_vertical_refresh_rate: bytes[17],
                            },

                            x => ExtendedTimingInformation::Unknown(x),
                        },
                    }
                }),
                x => Some(Descriptor::Unknown(x)),
            },
            _ => Some(Descriptor::DetailedTiming {
                pixel_clock_10khz: u16::from_le_bytes([byte0, byte1]),
                horizontal_active_pixels: u16::from_le_bytes([bytes[2], bytes[4] >> 4]),
                horizontal_blanking_pixels: u16::from_le_bytes([bytes[3], bytes[4] & 0xF]),
                vertical_active_lines: u16::from_le_bytes([bytes[5], bytes[7] >> 4]),
                vertical_blanking_lines: u16::from_le_bytes([bytes[6], bytes[7] & 0xF]),
                horizontal_front_porch_pixels: u16::from_le_bytes([bytes[8], bytes[11] >> 6]),
                horizontal_sync_pulse_width_pixels: u16::from_le_bytes([
                    bytes[9],
                    (bytes[11] >> 4) & 0b11,
                ]),
                vertical_front_porch_lines: (bytes[11] & 0b1100) << 2 | bytes[10] >> 4,
                vertical_sync_pulse_width_lines: (bytes[11] & 0b11) << 4 | bytes[10] & 0x0F,
                horizontal_image_size_mm: u16::from_le_bytes([bytes[12], bytes[14] >> 4]),
                vertical_image_size_mm: u16::from_le_bytes([bytes[13], bytes[14] & 0xF]),
                horizontal_border_pixels: bytes[15],
                vertical_border_lines: bytes[16],
                signal_interface_type: if bytes[17] & 0x80 == 0 {
                    SignalInterfaceType::NonInterlaced
                } else {
                    SignalInterfaceType::Interlaced
                },
                stereo_mode: unsafe {
                    core::mem::transmute(bytes[17] >> 4 & 0b110 | bytes[17] & 0b1)
                },
                sync: {
                    let bitmap = bytes[17];
                    match bitmap >> 4 & 1 {
                        0 => Sync::AnalogComposite {
                            bipolar: bitmap >> 3 & 1 == 1,
                            serrations_hsync_during_vsync: bitmap >> 2 & 1 == 1,
                            sync_on_red_and_blue_lines_additionally_to_green: bitmap >> 1 & 1
                                == 1,
                        },
                        _ => match bitmap >> 3 & 1 {
                            0 => Sync::DigitalComposite {
                                serrations_hsync_during_vsync: bitmap >> 2 & 1 == 1,
                                horizontal_sync_polarity: if bitmap >> 1 & 1 == 1 {
                                    Polarity::Positive
                                } else {
                                    Polarity::Negative
                                },
                            },
                            _ => Sync::DigitalSeparate {
                                vertical_sync_polarity: if bitmap >> 2 & 1 == 1 {
                                    Polarity::Positive
                                } else {
                                    Polarity::Negative
                                },
                                horizontal_sync_polarity: if bitmap >> 1 & 1 == 1 {
                                    Polarity::Positive
                                } else {
                                    Polarity::Negative
                                },
                            },
                        },
                    }
                },
            }),
        }
    }
}

pub struct DescriptorIterator<'a> (&'a [u8]);

impl Iterator for DescriptorIterator<'_> {
    type Item = Descriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() < 18 {
            return None;
        }
        let (head, tail) = self.0.split_at(18);
        self.0 = tail;
        Descriptor::from_bytes(head)
    }
}

impl EdidBlock {
    pub fn test_block() -> Self {
        let edid0: [u8; 128] = [
            0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x05, 0xe3, 0x09, 0x19, 0x01, 0x01,
            0x01, 0x01, 0x00, 0x14, 0x01, 0x03, 0x80, 0x40, 0x24, 0x78, 0x0a, 0x5d, 0x95, 0xa3,
            0x59, 0x53, 0xa0, 0x27, 0x0f, 0x50, 0x54, 0xaf, 0xce, 0x00, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x1b, 0x21,
            0x50, 0xa0, 0x51, 0x00, 0x1e, 0x30, 0x48, 0x88, 0x35, 0x00, 0x80, 0x68, 0x21, 0x00,
            0x00, 0x18, 0x02, 0x3a, 0x80, 0xd0, 0x72, 0x38, 0x2d, 0x40, 0x10, 0x2c, 0x45, 0x80,
            0x80, 0x68, 0x21, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0xfc, 0x00, 0x4c, 0x45, 0x31,
            0x39, 0x4b, 0x30, 0x39, 0x37, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0xfd,
            0x00, 0x38, 0x4c, 0x1e, 0x53, 0x11, 0x00, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x01, 0x3c,
        ];
        EdidBlock(edid0)
    }

    pub fn manufacturer_id(&self) -> [u8; 3] {
        let mut letters = [b'@'; 3];
        letters[0] += self.0[8] >> 2 & 0b11111;
        letters[1] += ((self.0[8] << 3) & 0b11000) | ((self.0[9] >> 5) & 0b00111);
        letters[2] += self.0[9] & 0b11111;
        letters
    }

    pub fn manufacturer_product_code(&self) -> u16 {
        u16::from_le_bytes(self.0[10..=11].try_into().unwrap())
    }

    pub fn serial_number(&self) -> u32 {
        u32::from_le_bytes(self.0[12..=15].try_into().unwrap())
    }

    pub fn week_of_manufacture(&self) -> u8 {
        self.0[16]
    }

    pub fn year_of_manufacture(&self) -> u16 {
        self.0[17] as u16 + 1990
    }

    pub fn edid_version(&self) -> u8 {
        self.0[18]
    }

    pub fn edid_revision(&self) -> u8 {
        self.0[19]
    }

    pub fn video_input_parameter(&self) -> VideoInputParameter {
        let byte = self.0[20];
        if byte & 0x80 == 0x80 {
            unsafe {
                VideoInputParameter::Digital {
                    bit_depth: core::mem::transmute((byte >> 4) & 0b111),
                    video_interface: core::mem::transmute(byte & 0b1111),
                }
            }
        } else {
            unsafe {
                VideoInputParameter::Analog {
                    white_and_sync_levels: core::mem::transmute((byte >> 5) & 0b11),
                    blank_to_blank_setup_expected: byte & 0b10000 != 0,
                    separate_sync_supported: byte & 0b01000 != 0,
                    composite_sync_on_h_supported: byte & 0b00100 != 0,
                    sync_on_green_supported: byte & 0b00010 != 0,
                    vsync_pulse_serrated: byte & 0b00001 != 0,
                }
            }
        }
    }

    pub fn screen_geometry(&self) -> ScreenGeometry {
        let bytes: [u8; 2] = self.0[21..=22].try_into().unwrap();
        match bytes {
            [0, 0] => ScreenGeometry::Undefined,
            [h, 0] => ScreenGeometry::Landscape(h),
            [0, v] => ScreenGeometry::Portrait(v),
            [h, v] => ScreenGeometry::SizeInCentimeters { h, v },
        }
    }

    pub fn gamma(&self) -> Option<f32> {
        match self.0[23] {
            255 => None,
            val => Some((val as f32 + 100.0) / 100.0),
        }
    }

    pub fn supported_features(&self) -> SupportedFeatures {
        let byte = self.0[24];
        let is_digital = self.0[20] & 0x80 != 0;
        SupportedFeatures {
            dpms_standby_supported: byte & 0x80 != 0,
            dpms_suspend_supported: byte & 0x40 != 0,
            dpms_active_off_supported: byte & 0x20 != 0,
            display_type: unsafe {
                if is_digital {
                    DisplayType::Digital(core::mem::transmute((byte >> 3) & 0b11))
                } else {
                    DisplayType::Analog(core::mem::transmute((byte >> 3) & 0b11))
                }
            },
            s_rgb: byte & 0x04 != 0,
            preferred_timing_mode: byte & 0x02 != 0,
            continuous_timings: byte & 0x01 != 0,
        }
    }

    pub fn chromaticity_coordinates(&self) -> ChromaticityCoordinates {
        let lsb_byte = self.0[25];
        let red_x = (lsb_byte >> 6) as u16 & 0b11 | (self.0[27] as u16) << 2;
        let red_y = (lsb_byte >> 4) as u16 & 0b11 | (self.0[28] as u16) << 2;
        let green_x = (lsb_byte >> 2) as u16 & 0b11 | (self.0[29] as u16) << 2;
        let green_y = (lsb_byte >> 0) as u16 & 0b11 | (self.0[30] as u16) << 2;

        let lsb_byte = self.0[26];
        let blue_x = (lsb_byte >> 6) as u16 & 0b11 | (self.0[31] as u16) << 2;
        let blue_y = (lsb_byte >> 4) as u16 & 0b11 | (self.0[32] as u16) << 2;
        let white_x = (lsb_byte >> 2) as u16 & 0b11 | (self.0[33] as u16) << 2;
        let white_y = (lsb_byte >> 0) as u16 & 0b11 | (self.0[34] as u16) << 2;

        ChromaticityCoordinates {
            red: CIEPoint {
                x: red_x as f32 / 1024.0,
                y: red_y as f32 / 1024.0,
            },
            green: CIEPoint {
                x: green_x as f32 / 1024.0,
                y: green_y as f32 / 1024.0,
            },
            blue: CIEPoint {
                x: blue_x as f32 / 1024.0,
                y: blue_y as f32 / 1024.0,
            },
            white: CIEPoint {
                x: white_x as f32 / 1024.0,
                y: white_y as f32 / 1024.0,
            },
        }
    }

    pub fn common_timing_support(&self) -> CommonLegacyTimingSupport {
        let byte_low = self.0[35];
        let byte_mid = self.0[36];
        let byte_up = self.0[37];
        CommonLegacyTimingSupport {
            _720_400_70: byte_low & 0x80 != 0,
            _720_400_88: byte_low & 0x40 != 0,
            _640_480_60: byte_low & 0x20 != 0,
            _640_480_67: byte_low & 0x10 != 0,
            _640_480_72: byte_low & 0x08 != 0,
            _640_480_75: byte_low & 0x04 != 0,
            _800_600_56: byte_low & 0x02 != 0,
            _800_600_60: byte_low & 0x01 != 0,
            _800_600_72: byte_mid & 0x80 != 0,
            _800_600_75: byte_mid & 0x40 != 0,
            _832_624_75: byte_mid & 0x20 != 0,
            _1024_768_87_interlaced: byte_mid & 0x10 != 0,
            _1024_768_60: byte_mid & 0x08 != 0,
            _1024_768_70: byte_mid & 0x04 != 0,
            _1024_768_75: byte_mid & 0x02 != 0,
            _1280_1024_75: byte_mid & 0x01 != 0,
            _1152_870_75: byte_up & 0x80 != 0,
        }
    }

    pub fn standard_timing_information(&self) -> [Option<StandardTimingInformation>; 8] {
        let mut result = [None; 8];
        for i in 0..8 {
            let byte0 = self.0[38 + i * 2];
            let byte1 = self.0[39 + i * 2];
            result[i] = match (byte0, byte1) {
                (0x01, 0x01) => None,
                _ => Some(StandardTimingInformation {
                    x_resolution: (byte0 as u16 + 31) * 8,
                    image_aspect_ratio: unsafe { core::mem::transmute(byte1 >> 6) },
                    vertical_frequency: byte1 & 0x3F + 60,
                }),
            }
        }
        result
    }

    pub fn descriptors(&self) -> [Option<Descriptor>; 4] {
        let mut result = [None; 4];
        if let Some(bytes) = self.0.get(54..=125) {
            for (i, desc) in DescriptorIterator(bytes).enumerate() {
                result[i] = Some(desc);
            }
        }
        result
    }

    fn extension_len(&self) -> u8 {
        self.0[126]
    }
}

impl Debug for EdidBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EdidBlock")
            .field("Video Input Parameter", &self.video_input_parameter())
            .field("Screen Geometry", &self.screen_geometry())
            .field("Supported Features", &self.supported_features())
            .field("Chromaticity Coordinates", &self.chromaticity_coordinates())
            .field("Common Timing Support", &self.common_timing_support())
            .field(
                "Standard Timing Information",
                &self.standard_timing_information(),
            )
            .field("Descriptors", &self.descriptors())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
pub enum CtaDataBlock<'a> {
    None,
    Audio {
        audio_blocks: CtaAudioIterator<'a>,
    },
    Video {
        video_blocks: CtaVideoIterator<'a>,
    },
    VendorSpecific {
        ieee_registration_id: [u8; 3],
        payload: &'a [u8],
    },
    SpeakerAllocation {
        rear_left_and_right_center: bool,
        front_left_and_right_center: bool,
        rear_center: bool,
        rear_left_and_right: bool,
        front_center: bool,
        low_frequency_effects: bool,
        front_left_and_right: bool,
    },
    VesaDisplayTransferCharacteristic,
    VideoFormat,
    Extended,
}

impl<'a> CtaDataBlock<'a> {
    pub fn new(block: &'a [u8]) -> Self {
        let block_type = block[0] >> 5;
        let len = block[0] & 0x1f;
        let bytes = block.split_at(1).1.split_at(len as usize).0;
        match block_type {
            1 => Self::Audio {
                audio_blocks: CtaAudioIterator { bytes },
            },
            2 => Self::Video {
                video_blocks: CtaVideoIterator { bytes },
            },
            3 => {
                let (head, tail) = bytes.split_at(3);
                Self::VendorSpecific {
                    ieee_registration_id: head.try_into().unwrap(),
                    payload: tail,
                }
            }
            4 => Self::SpeakerAllocation {
                rear_left_and_right_center: bytes[0] & 0x40 != 0,
                front_left_and_right_center: bytes[0] & 0x20 != 0,
                rear_center: bytes[0] & 0x10 != 0,
                rear_left_and_right: bytes[0] & 0x08 != 0,
                front_center: bytes[0] & 0x04 != 0,
                low_frequency_effects: bytes[0] & 0x02 != 0,
                front_left_and_right: bytes[0] & 0x01 != 0,
            },
            5 => Self::VesaDisplayTransferCharacteristic,
            6 => Self::VideoFormat,
            7 => Self::Extended,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CtaDataBlockIterator<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for CtaDataBlockIterator<'a> {
    type Item = CtaDataBlock<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            None
        } else {
            let header = self.bytes.first().unwrap();
            let len = header & 0x1F;
            if len == 0 {
                None
            } else {
                let (block, rest) = self.bytes.split_at(len as usize + 1);
                self.bytes = rest;
                Some(CtaDataBlock::new(block))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Reserved,
    LinearPulseCodeModulation,
    Ac3,
    Mpeg1Layer1_2,
    Mp3Mpeg1Layer3,
    Mpeg2,
    AacLc,
    Dts,
    Atrac,
    Sacd,
    DdPlus,
    DtsHd,
    MatMlpDolbyTrueHd,
    DstAudio,
    WmaPro,
    Extension,
}

#[derive(Debug, Clone, Copy)]
pub struct CtaShortAudioDescriptor {
    audio_format: AudioFormat,
    num_channels: u8,
    sr_192: bool,
    sr_176: bool,
    sr_96: bool,
    sr_88: bool,
    sr_48: bool,
    sr_44_1: bool,
    sr_32: bool,
}

impl CtaShortAudioDescriptor {
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        CtaShortAudioDescriptor {
            audio_format: unsafe { core::mem::transmute(bytes[0] >> 3) },
            num_channels: (bytes[0] & 0b111) + 1,
            sr_192: bytes[1] & 0x40 != 0,
            sr_176: bytes[1] & 0x20 != 0,
            sr_96: bytes[1] & 0x10 != 0,
            sr_88: bytes[1] & 0x08 != 0,
            sr_48: bytes[1] & 0x04 != 0,
            sr_44_1: bytes[1] & 0x02 != 0,
            sr_32: bytes[1] & 0x01 != 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CtaAudioIterator<'a> {
    bytes: &'a [u8],
}

impl Iterator for CtaAudioIterator<'_> {
    type Item = CtaShortAudioDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            None
        } else {
            let (block, rest) = self.bytes.split_at(3);
            self.bytes = rest;
            Some(CtaShortAudioDescriptor::from_bytes(
                block.try_into().unwrap(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CtaShortVideoDescriptor {
    native: bool,
    video_identification_code: u8,
}

impl CtaShortVideoDescriptor {
    pub fn from_bytes(bytes: [u8; 1]) -> Self {
        CtaShortVideoDescriptor {
            native: bytes[0] & 0x80 != 0,
            video_identification_code: bytes[0] & 0x7F,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CtaVideoIterator<'a> {
    bytes: &'a [u8],
}

impl Iterator for CtaVideoIterator<'_> {
    type Item = CtaShortVideoDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            None
        } else {
            let (block, rest) = self.bytes.split_at(1);
            self.bytes = rest;
            Some(CtaShortVideoDescriptor::from_bytes(
                block.try_into().unwrap(),
            ))
        }
    }
}

pub struct CtaExtensionBlock([u8; 128]);
impl CtaExtensionBlock {
    pub fn test_block() -> Self {
        let edid1: [u8; 128] = [
            0x02, 0x03, 0x26, 0x70, 0x4e, 0x13, 0x04, 0x1f, 0x10, 0x20, 0x21, 0x22, 0x14, 0x05,
            0x11, 0x02, 0x15, 0x06, 0x01, 0x26, 0x09, 0x07, 0x03, 0x15, 0x07, 0x50, 0x83, 0x01,
            0x00, 0x00, 0x67, 0x03, 0x0c, 0x00, 0x20, 0x00, 0xb8, 0x2d, 0x01, 0x1d, 0x80, 0x3e,
            0x73, 0x38, 0x2d, 0x40, 0x7e, 0x2c, 0x45, 0x80, 0x80, 0x68, 0x21, 0x00, 0x00, 0x1e,
            0x01, 0x1d, 0x80, 0xd0, 0x72, 0x1c, 0x16, 0x20, 0x10, 0x2c, 0x25, 0x80, 0x80, 0x68,
            0x21, 0x00, 0x00, 0x9e, 0x01, 0x1d, 0x00, 0xbc, 0x52, 0xd0, 0x1e, 0x20, 0xb8, 0x28,
            0x55, 0x40, 0x80, 0x68, 0x21, 0x00, 0x00, 0x1e, 0x8c, 0x0a, 0xd0, 0x90, 0x20, 0x40,
            0x31, 0x20, 0x0c, 0x40, 0x55, 0x00, 0x90, 0x2c, 0x11, 0x00, 0x00, 0x18, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x3b,
        ];
        Self(edid1)
    }

    pub fn cta_revision(&self) -> u8 {
        self.0[1]
    }

    pub fn dtd_offset(&self) -> u8 {
        self.0[2]
    }

    pub fn support_underscan(&self) -> bool {
        self.0[3] & 0x80 != 0
    }

    pub fn support_basic_audio(&self) -> bool {
        self.0[3] & 0x40 != 0
    }

    pub fn support_ycbcr_444(&self) -> bool {
        self.0[3] & 0x20 != 0
    }

    pub fn support_ycbcr_422(&self) -> bool {
        self.0[3] & 0x10 != 0
    }

    pub fn native_format_count(&self) -> u8 {
        self.0[3] & 0xF
    }

    pub fn data_blocks(&self) -> CtaDataBlockIterator {
        let front = self.0.split_at(self.dtd_offset() as usize).0;
        let data_block_slice = front.split_at(4).1;

        CtaDataBlockIterator {
            bytes: data_block_slice,
        }
    }

    pub fn descriptors(&self) -> DescriptorIterator {
        let bytes = self.0.get(self.dtd_offset() as usize..);
        DescriptorIterator(bytes.unwrap_or_default()) 
    }
}

impl Debug for CtaExtensionBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CtaExtensionBlock")
            .field("Support underscan", &self.support_underscan())
            .field("Support basic_audio", &self.support_basic_audio())
            .field("Support ycbcr_444", &self.support_ycbcr_444())
            .field("Support ycbcr_422", &self.support_ycbcr_422())
            .field("Native format count", &self.native_format_count());
        f.debug_set().entries(self.data_blocks());
        f.debug_set().entries(self.descriptors()).finish()
    }
}

impl Debug for CtaDataBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Audio { audio_blocks } => f.debug_list().entries(*audio_blocks).finish(),
            Self::Video { video_blocks } => f.debug_list().entries(*video_blocks).finish(),
            Self::VendorSpecific {
                ieee_registration_id,
                payload,
            } => f
                .debug_struct("VendorSpecific")
                .field("ieee_registration_id", ieee_registration_id)
                .field("payload", payload)
                .finish(),
            Self::SpeakerAllocation {
                rear_left_and_right_center,
                front_left_and_right_center,
                rear_center,
                rear_left_and_right,
                front_center,
                low_frequency_effects,
                front_left_and_right,
            } => f
                .debug_struct("SpeakerAllocation")
                .field("rear_left_and_right_center", rear_left_and_right_center)
                .field("front_left_and_right_center", front_left_and_right_center)
                .field("rear_center", rear_center)
                .field("rear_left_and_right", rear_left_and_right)
                .field("front_center", front_center)
                .field("low_frequency_effects", low_frequency_effects)
                .field("front_left_and_right", front_left_and_right)
                .finish(),
            Self::VesaDisplayTransferCharacteristic => {
                write!(f, "VesaDisplayTransferCharacteristic")
            }
            Self::VideoFormat => write!(f, "VideoFormat"),
            Self::Extended => write!(f, "Extended"),
        }
    }
}

pub struct EdidIterator {
    block_num: u8,
    block_total: u8,
}

impl EdidIterator {
    pub fn new() -> Self {
        Self {
            block_num: 0,
            block_total: 1,
        }
    }
}

impl core::iter::Iterator for EdidIterator {
    type Item = Edid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.block_num == self.block_total {
            return None;
        }
        use mailbox::PropertyMessageRequest::*;
        let mut mb = mailbox::Mailbox::<256>::new();
        mb.push_tag(GetEdidBlock {
            block_number: self.block_num as u32,
        });
        mb.push_tag(Null);
        if mb.submit_messages(8).is_ok() {
            let (_block_number, status, data): (u32, u32, [u8; 128]) = mb.pop_values();
            if status == 0 {
                let block = Edid::from_bytes(&data);
                match &block {
                    Edid::Edid(edid) => {
                        self.block_total = 1 + edid.extension_len();
                    }
                    _ => {}
                }
                self.block_num += 1;
                Some(block)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct MockEdidIterator {
    block_num: u8,
    block_total: u8,
}

impl MockEdidIterator {
    pub fn new() -> Self {
        Self {
            block_num: 0,
            block_total: 2,
        }
    }
}

impl core::iter::Iterator for MockEdidIterator {
    type Item = Edid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.block_num == self.block_total {
            return None;
        }

        self.block_num += 1;
        match self.block_num {
            1 => Some(Edid::Edid(EdidBlock::test_block())),
            2 => Some(Edid::CtaExtensionRev3(CtaExtensionBlock::test_block())),
            _ => None,
        }
    }
}
