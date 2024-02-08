#![no_std]

pub mod bcd;
pub mod bitfield;
pub mod buffer;
pub mod drawing;
pub mod format;
pub mod parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
