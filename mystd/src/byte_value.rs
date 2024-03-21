pub struct ByteValue(pub u64);

impl ByteValue {
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub const fn from_kibi(kibi: u64) -> Self {
        Self(kibi * 1024)
    }

    pub const fn from_mibi(mibi: u64) -> Self {
        Self(mibi * 1024 * 1024)
    }

    pub const fn from_gibi(gibi: u64) -> Self {
        Self(gibi * 1024 * 1024 * 1024)
    }

    pub const fn as_bytes(self) -> u64 {
        self.0
    }

    pub const fn as_kibi(self) -> u64 {
        self.0 >> 10
    }

    pub const fn as_mibi(self) -> u64 {
        self.0 >> 20
    }

    pub const fn as_gibi(self) -> u64 {
        self.0 >> 30
    }
}

impl core::fmt::Display for ByteValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 == 0 {
            return f.write_str("0 B");
        }
        const MASK: u64 = (1 << 10) - 1;
        const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
        let mut values = [0_u64; 7];
        let mut index = 0;
        let mut remaining = self.0;
        loop {
            let val = remaining & MASK;
            values[index] = val;
            remaining >>= 10;
            if remaining == 0 {
                write!(f, "{} {}", values[index], UNITS[index])?;
                break;
            }
            index += 1;
        }

        while index > 0 {
            index -= 1;
            let val = values[index];
            if val != 0 {
                write!(f, " {} {}", val, UNITS[index])?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::ring::RingArray;

    use super::*;

    #[test]
    fn byte_value_fmt_works() {
        use core::fmt::Write;

        let mut buf: RingArray<u8, 256> = RingArray::new();
        write!(buf, "{}", ByteValue(0)).expect("should work");
        assert_eq!("0 B", buf.to_str().unwrap());

        buf.clear();
        write!(buf, "{}", ByteValue(1024)).expect("should work");
        assert_eq!("1 KiB", buf.to_str().unwrap());

        buf.clear();
        write!(buf, "{}", ByteValue(1024 * 1024 + 1028)).expect("should work");
        assert_eq!("1 MiB 1 KiB 4 B", buf.to_str().unwrap());
    }
}
