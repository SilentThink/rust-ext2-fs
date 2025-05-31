//!
//! # Ext2 文件系统


pub mod core;
pub mod constant;
mod func;

pub use crate::fs::core::*;

#[test]
fn test_bits() {
    let bits: u8 = 0b1000_0000;
    assert_eq!(bits, 0b1000_0000);
    assert_eq!(bits >> 1, 0b0100_0000);
    assert_eq!(bits >> 2, 0b0010_0000);
    assert_eq!(bits >> 3, 0b0001_0000);
    assert_eq!(bits >> 7, 0b0000_0001);
}
