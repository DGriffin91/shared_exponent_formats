pub mod custom_shared_format;
pub mod evaluate;
pub mod rgb9e5;
pub mod xyz13e6;
pub mod xyz14e3;
pub mod xyz18e7;
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

pub fn nan_to_zero64(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value
    }
}
