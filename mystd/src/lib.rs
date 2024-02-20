#![no_std]

pub mod bcd;
pub mod bitfield;
pub mod buffer;
pub mod drawing;
pub mod format;
pub mod parse;
pub mod fixed_point;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
