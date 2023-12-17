#![cfg_attr(target_arch = "spirv", no_std)]

pub mod custom_shared_format;
#[cfg(not(target_arch = "spirv"))]
pub mod evaluate;
pub mod rgb9e5;
pub mod xyz13e6;
pub mod xyz14e3;
pub mod xyz18e7;
#[cfg(not(target_arch = "spirv"))]
pub mod xyz18e7_f64test;
pub mod xyz8e5;
pub mod xyz9e2;

pub fn nan_to_zero(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

#[cfg(not(target_arch = "spirv"))]
pub fn nan_to_zero64(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

// https://discord.com/channels/750717012564770887/750717499737243679/1185773394642350131
#[cfg(target_arch = "spirv")]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

#[cfg(target_arch = "spirv")]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}
