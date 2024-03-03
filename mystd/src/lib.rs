#![no_std]

pub mod bcd;
pub mod bitfield;
pub mod collections;
pub mod drawing;
pub mod fixed_point;
pub mod format;
pub mod parse;

#[cfg(test)]
mod tests {
    use super::*;



    

    #[test]
    fn stride_works() {
        let a = Dma2dStride::new(-4, -5);
        assert_eq!(-4, a.source);
        assert_eq!(-5, a.destination);
    }
}
