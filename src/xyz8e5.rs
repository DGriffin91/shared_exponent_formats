#[cfg(target_arch = "spirv")]
use crate::max;
#[cfg(not(target_arch = "spirv"))]
use core::cmp::max;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::nan_to_zero;

pub const NAME: &str = "xyz8e5";
pub const BYTES: u8 = 4;
pub const SIGNED: bool = true;

pub const XYZ8E5_EXPONENT_BITS: i32 = 5;
pub const XYZ8E5_MANTISSA_BITS: i32 = 8;
pub const XYZ8E5_MANTISSA_BITSU: u32 = 8;
pub const XYZ8E5_EXP_BIAS: i32 = 15;
pub const XYZ8E5_MAX_VALID_BIASED_EXP: i32 = 31;

/*
pub const MAX_XYZ8E5_EXP: i32 = 16;
pub const XYZ8E5_MANTISSA_VALUES: i32 = 256;
pub const MAX_XYZ8E5_MANTISSA: i32 = 255;
pub const MAX_XYZ8E5_MANTISSAU: u32 = 255;
pub const MAX_XYZ8E5: f32 = 65280.0;
pub const EPSILON_XYZ8E5: f32 = 1.1920929e-7;
*/

pub const MAX_XYZ8E5_EXP: i32 = XYZ8E5_MAX_VALID_BIASED_EXP - XYZ8E5_EXP_BIAS;
pub const XYZ8E5_MANTISSA_VALUES: i32 = 1 << XYZ8E5_MANTISSA_BITS;
pub const MAX_XYZ8E5_MANTISSA: i32 = XYZ8E5_MANTISSA_VALUES - 1;
pub const MAX_XYZ8E5_MANTISSAU: u32 = (XYZ8E5_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ8E5: f32 =
    (MAX_XYZ8E5_MANTISSA as f32) / XYZ8E5_MANTISSA_VALUES as f32 * (1 << MAX_XYZ8E5_EXP) as f32;
pub const EPSILON_XYZ8E5: f32 =
    (1.0 / XYZ8E5_MANTISSA_VALUES as f32) / (1 << XYZ8E5_EXP_BIAS) as f32;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz8e5(xyz: [f32; 3]) -> u32 {
    let xsign = xyz[0].is_sign_negative() as u32;
    let ysign = xyz[1].is_sign_negative() as u32;
    let zsign = xyz[2].is_sign_negative() as u32;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ8E5);
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ8E5);
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ8E5);

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        max(-XYZ8E5_EXP_BIAS - 1, maxxyz.log2().floor() as i32) + 1 + XYZ8E5_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ8E5_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ8E5_EXP_BIAS - XYZ8E5_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxxyz / denom + 0.5).floor() as i32;
    if maxm == MAX_XYZ8E5_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= XYZ8E5_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_XYZ8E5_MANTISSA);
    }

    let xm = (xc / denom + 0.5).floor() as i32;
    let ym = (yc / denom + 0.5).floor() as i32;
    let zm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(xm <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(ym <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(zm <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(xm >= 0);
    debug_assert!(ym >= 0);
    debug_assert!(zm >= 0);

    debug_assert_eq!(xm as u32, xm as u32 & MAX_XYZ8E5_MANTISSAU);
    debug_assert_eq!(ym as u32, ym as u32 & MAX_XYZ8E5_MANTISSAU);
    debug_assert_eq!(zm as u32, zm as u32 & MAX_XYZ8E5_MANTISSAU);

    let xm = xm as u32 | xsign << 8;
    let ym = ym as u32 | ysign << 8;
    let zm = zm as u32 | zsign << 8;
    let exp_shared = exp_shared as u32;

    #[allow(clippy::identity_op)]
    let ret = (exp_shared << 27) | (zm << 18) | (ym << 9) | (xm << 0);

    ret
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
pub fn xyz8e5_to_vec3(v: u32) -> [f32; 3] {
    let exponent = bitfield_extract(v, 27, XYZ8E5_EXPONENT_BITS as u32) as i32
        - XYZ8E5_EXP_BIAS
        - XYZ8E5_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    // Extract both the mantissa and sign at the same time.
    let xb = bitfield_extract(v, 0, XYZ8E5_MANTISSA_BITSU + 1);
    let yb = bitfield_extract(v, 9, XYZ8E5_MANTISSA_BITSU + 1);
    let zb = bitfield_extract(v, 18, XYZ8E5_MANTISSA_BITSU + 1);

    // xb & 0xFFu masks out for just the mantissa
    let xm = ((xb & 0xFFu32) as f32).to_bits();
    let ym = ((yb & 0xFFu32) as f32).to_bits();
    let zm = ((zb & 0xFFu32) as f32).to_bits();

    // xb & 0x100u << 23u masks out just the sign bit and shifts it over
    // to the corresponding IEEE 754 sign location
    [
        f32::from_bits(xm | (xb & 0x100u32) << 23u32) * scale,
        f32::from_bits(ym | (yb & 0x100u32) << 23u32) * scale,
        f32::from_bits(zm | (zb & 0x100u32) << 23u32) * scale,
    ]
}

#[cfg(test)]
pub mod tests {

    use glam::Vec3;

    use crate::evaluate::test_util::{Report, DEFUALT_ITERATIONS};

    use super::*;

    #[test]
    fn test_accuracy() {
        for (dist, max) in [
            (0.01, 5.28e-5),
            (0.1, 4.23e-4),
            (1.0, 5.82e-3),
            (10.0, 5.40e-2),
            (100.0, 4.33e-1),
            (1000.0, 3.46),
        ] {
            let r = Report::new(dist, DEFUALT_ITERATIONS, true, |v| {
                xyz8e5_to_vec3(vec3_to_xyz8e5(v.into())).into()
            });
            dbg!(r.max_dist, max);
            assert!(r.max_dist < max);
        }
    }

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::NEG_ONE,
            xyz8e5_to_vec3(vec3_to_xyz8e5((Vec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::ONE,
            xyz8e5_to_vec3(vec3_to_xyz8e5((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ8E5),
            xyz8e5_to_vec3(vec3_to_xyz8e5(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ8E5),
            xyz8e5_to_vec3(vec3_to_xyz8e5((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ8E5),
            xyz8e5_to_vec3(vec3_to_xyz8e5(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ8E5),
            xyz8e5_to_vec3(vec3_to_xyz8e5((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz8e5_to_vec3(vec3_to_xyz8e5((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz8e5_to_vec3(vec3_to_xyz8e5((Vec3::NAN).into())).into()
        );
    }
}
