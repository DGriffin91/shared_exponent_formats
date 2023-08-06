use crate::nan_to_zero;

pub const NAME: &str = "rgb9e5";
pub const BYTES: u8 = 4;
pub const SIGNED: bool = false;

pub const RGB9E5_EXPONENT_BITS: i32 = 5;
pub const RGB9E5_MANTISSA_BITS: i32 = 9;
pub const RGB9E5_EXP_BIAS: i32 = 15;
pub const RGB9E5_MAX_VALID_BIASED_EXP: i32 = 31;

/*
pub const MAX_RGB9E5_EXP: i32 = 16;
pub const RGB9E5_MANTISSA_VALUES: i32 = 512;
pub const MAX_RGB9E5_MANTISSA: i32 = 511;
pub const MAX_RGB9E5: f32 = 65408.0;
pub const EPSILON_RGB9E5: f32 = 5.9604645e-8;
 */

pub const MAX_RGB9E5_EXP: i32 = RGB9E5_MAX_VALID_BIASED_EXP - RGB9E5_EXP_BIAS;
pub const RGB9E5_MANTISSA_VALUES: i32 = 1 << RGB9E5_MANTISSA_BITS;
pub const MAX_RGB9E5_MANTISSA: i32 = RGB9E5_MANTISSA_VALUES - 1;
#[allow(dead_code)]
pub const MAX_RGB9E5: f32 =
    (MAX_RGB9E5_MANTISSA as f32) / RGB9E5_MANTISSA_VALUES as f32 * (1 << MAX_RGB9E5_EXP) as f32;
#[allow(dead_code)]
pub const EPSILON_RGB9E5: f32 =
    (1.0 / RGB9E5_MANTISSA_VALUES as f32) / (1 << RGB9E5_EXP_BIAS) as f32;

// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_rgb9e5(rgb: [f32; 3]) -> u32 {
    let rc = nan_to_zero(rgb[0]).clamp(0.0, MAX_RGB9E5);
    let gc = nan_to_zero(rgb[1]).clamp(0.0, MAX_RGB9E5);
    let bc = nan_to_zero(rgb[2]).clamp(0.0, MAX_RGB9E5);

    let maxrgb = rc.max(gc).max(bc);
    let mut exp_shared =
        (-RGB9E5_EXP_BIAS - 1).max(maxrgb.log2().floor() as i32) + 1 + RGB9E5_EXP_BIAS;

    debug_assert!(exp_shared <= RGB9E5_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - RGB9E5_EXP_BIAS - RGB9E5_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxrgb / denom + 0.5).floor() as i32;
    if maxm == MAX_RGB9E5_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= RGB9E5_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_RGB9E5_MANTISSA);
    }

    let rm = (rc / denom + 0.5).floor() as i32;
    let gm = (gc / denom + 0.5).floor() as i32;
    let bm = (bc / denom + 0.5).floor() as i32;

    debug_assert!(rm <= MAX_RGB9E5_MANTISSA);
    debug_assert!(gm <= MAX_RGB9E5_MANTISSA);
    debug_assert!(bm <= MAX_RGB9E5_MANTISSA);
    debug_assert!(rm >= 0);
    debug_assert!(gm >= 0);
    debug_assert!(bm >= 0);

    let rm = rm as u32;
    let gm = gm as u32;
    let bm = bm as u32;
    let exp_shared = exp_shared as u32;

    #[allow(clippy::identity_op)]
    let ret = (exp_shared << 27) | (bm << 18) | (gm << 9) | (rm << 0);

    ret
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
pub fn rgb9e5_to_vec3(v: u32) -> [f32; 3] {
    let exponent = bitfield_extract(v, 27, RGB9E5_EXPONENT_BITS as u32) as i32
        - RGB9E5_EXP_BIAS
        - RGB9E5_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    [
        bitfield_extract(v, 0, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
        bitfield_extract(v, 9, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
        bitfield_extract(v, 18, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
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
            (0.01, 2.64e-5),
            (0.1, 2.11e-4),
            (1.0, 2.91e-3),
            (10.0, 2.70e-2),
            (100.0, 2.16e-1),
            (1000.0, 1.73),
        ] {
            let r = Report::new(dist, DEFUALT_ITERATIONS, false, |v| {
                rgb9e5_to_vec3(vec3_to_rgb9e5(v.into())).into()
            });
            assert!(r.max_dist < max);
        }
    }

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::ONE,
            rgb9e5_to_vec3(vec3_to_rgb9e5((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_RGB9E5),
            rgb9e5_to_vec3(vec3_to_rgb9e5(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            rgb9e5_to_vec3(vec3_to_rgb9e5((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_RGB9E5),
            rgb9e5_to_vec3(vec3_to_rgb9e5(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            rgb9e5_to_vec3(vec3_to_rgb9e5((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            rgb9e5_to_vec3(vec3_to_rgb9e5((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            rgb9e5_to_vec3(vec3_to_rgb9e5((Vec3::NAN).into())).into()
        );
    }
}
