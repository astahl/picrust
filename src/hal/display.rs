use core::str::Bytes;

use crate::peripherals::mailbox;

pub struct EdidBlock ([u8;128]);


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
    Plus0_7Minus0EVC
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
    }
}

#[derive(Debug)]
pub enum ScreenGeometry {
    Undefined,
    Landscape(u8),
    Portrait(u8),
    SizeInCentimeters{h: u8, v: u8}
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
    Analog (DisplayTypeAnalog),
    Digital (DisplayTypeDigital)
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
pub struct CIEPoint{ x: f32, y: f32 }

#[derive(Debug, Default)]
pub struct ChromaticityCoordinates {
    red: CIEPoint,
    green: CIEPoint,
    blue: CIEPoint,
    white: CIEPoint
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


impl EdidBlock {

    pub fn test_block0() -> Self {
        let edid0: [u8; 128] = [
            0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00,
            0x05, 0xe3, 0x09, 0x19, 0x01, 0x01, 0x01, 0x01,
            0x00, 0x14, 0x01, 0x03, 0x80, 0x40, 0x24, 0x78,
            0x0a, 0x5d, 0x95, 0xa3, 0x59, 0x53, 0xa0, 0x27,
            0x0f, 0x50, 0x54, 0xaf, 0xce, 0x00, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x1b, 0x21,
            0x50, 0xa0, 0x51, 0x00, 0x1e, 0x30, 0x48, 0x88, 
            0x35, 0x00, 0x80, 0x68, 0x21, 0x00, 0x00, 0x18, 
            0x02, 0x3a, 0x80, 0xd0, 0x72, 0x38, 0x2d, 0x40, 
            0x10, 0x2c, 0x45, 0x80, 0x80, 0x68, 0x21, 0x00, 
            0x00, 0x1e, 0x00, 0x00, 0x00, 0xfc, 0x00, 0x4c, 
            0x45, 0x31, 0x39, 0x4b, 0x30, 0x39, 0x37, 0x0a, 
            0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0xfd, 
            0x00, 0x38, 0x4c, 0x1e, 0x53, 0x11, 0x00, 0x0a, 
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x01, 0x3c
            ];
        EdidBlock(edid0)
    }

    pub fn checksum_ok(&self) -> bool {
        let mut sum: u8 = 0;
        for byte in self.0 {
            sum = sum.wrapping_add(byte);
        }
        sum == 0
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
                    video_interface: core::mem::transmute(byte & 0b1111) 
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
                    vsync_pulse_serrated: byte & 0b00001 != 0 }
            }
        }
    }

    pub fn screen_geometry(&self) -> ScreenGeometry {
        match (self.0[21], self.0[22]) {
            (0, 0) => ScreenGeometry::Undefined,
            (h, 0) => ScreenGeometry::Landscape(h),
            (0, v) => ScreenGeometry::Portrait(v),
            (h, v) => ScreenGeometry::SizeInCentimeters { h, v }
        }
    }

    pub fn gamma(&self) -> Option<f32> {
        match self.0[23] {
            255 => None,
            val => Some((val as f32 + 100.0) / 100.0)
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
                }
                else {
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
            red: CIEPoint { x: red_x as f32 / 1024.0, y: red_y as f32 / 1024.0 },
            green: CIEPoint { x: green_x as f32 / 1024.0, y: green_y as f32 / 1024.0 },
            blue: CIEPoint { x: blue_x as f32 / 1024.0, y: blue_y as f32 / 1024.0 },
            white: CIEPoint { x: white_x as f32 / 1024.0, y: white_y as f32 / 1024.0 },
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
            _1152_870_75: byte_up & 0x80 != 0 }
    }
}

pub struct EdidIterator {
    block_num: u32,
    done: bool
}

impl EdidIterator {
    pub fn new() -> Self {
        Self {block_num: 0, done: false}
    }
}

impl core::iter::Iterator for EdidIterator {
    type Item = (u32, [u8;128]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        use mailbox::PropertyMessageRequest::*;
        let mut mb = mailbox::Mailbox::<256>::new();
        mb.push_tag(GetEdidBlock { block_number: self.block_num });
        mb.push_tag(Null);
        if mb.submit_messages(8).is_ok() {
            let (block_number, status, data): (u32, u32, [u8; 128]) = mb.pop_values();
            if status == 0 {
                self.block_num += 1;
            } else {
                self.done = true;
            }
            Some((block_number, data))
        } else {
            self.done = true;
            None
        }
    }
}
