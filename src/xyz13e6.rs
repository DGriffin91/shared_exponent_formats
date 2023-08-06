use crate::nan_to_zero;

pub const NAME: &str = "xyz13e6";
pub const BYTES: u8 = 6;
pub const SIGNED: bool = true;

pub const XYZ13E6_EXPONENT_BITS: i32 = 6;
pub const XYZ13E6_MANTISSA_BITS: i32 = 13;
pub const XYZ13E6_MANTISSA_BITSU: u32 = 13;
pub const XYZ13E6_EXP_BIAS: i32 = 31;
pub const XYZ13E6_MAX_VALID_BIASED_EXP: i32 = 63;

/*
pub const MAX_XYZ13E6_EXP: i32 = 32;
pub const XYZ13E6_MANTISSA_VALUES: i32 = 8192;
pub const MAX_XYZ13E6_MANTISSA: i32 = 8191;
pub const MAX_XYZ13E6_MANTISSAU: u32 = 8191;
pub const MAX_XYZ13E6: f32 = 4294443000.0;
pub const EPSILON_XYZ13E6: f32 = 5.684342e-14;
*/

pub const MAX_XYZ13E6_EXP: u64 = XYZ13E6_MAX_VALID_BIASED_EXP as u64 - XYZ13E6_EXP_BIAS as u64;
pub const XYZ13E6_MANTISSA_VALUES: i32 = 1 << XYZ13E6_MANTISSA_BITS;
pub const MAX_XYZ13E6_MANTISSA: i32 = XYZ13E6_MANTISSA_VALUES - 1;
pub const MAX_XYZ13E6_MANTISSAU: u32 = (XYZ13E6_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ13E6: f32 = (MAX_XYZ13E6_MANTISSA as f32) / XYZ13E6_MANTISSA_VALUES as f32
    * (1u128 << MAX_XYZ13E6_EXP) as f32;
pub const EPSILON_XYZ13E6: f32 =
    (1.0 / XYZ13E6_MANTISSA_VALUES as f32) / (1u64 << XYZ13E6_EXP_BIAS) as f32;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz13e6(xyz: [f32; 3]) -> (u32, u16) {
    let xsign = xyz[0].is_sign_negative() as u16;
    let ysign = xyz[1].is_sign_negative() as u16;
    let zsign = xyz[2].is_sign_negative() as u16;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ13E6);
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ13E6);
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ13E6);

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ13E6_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ13E6_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ13E6_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxxyz / denom + 0.5).floor() as i32;
    if maxm == MAX_XYZ13E6_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= XYZ13E6_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_XYZ13E6_MANTISSA);
    }

    let xm = (xc / denom + 0.5).floor() as i32;
    let ym = (yc / denom + 0.5).floor() as i32;
    let zm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(xm <= MAX_XYZ13E6_MANTISSA);
    debug_assert!(ym <= MAX_XYZ13E6_MANTISSA);
    debug_assert!(zm <= MAX_XYZ13E6_MANTISSA);
    debug_assert!(xm >= 0);
    debug_assert!(ym >= 0);
    debug_assert!(zm >= 0);

    debug_assert_eq!(xm as u32, xm as u32 & MAX_XYZ13E6_MANTISSAU);
    debug_assert_eq!(ym as u32, ym as u32 & MAX_XYZ13E6_MANTISSAU);
    debug_assert_eq!(zm as u32, zm as u32 & MAX_XYZ13E6_MANTISSAU);

    let exp_shared = exp_shared as u32;

    let xm = xm as u32;
    let ym = ym as u32;
    let zm = zm as u16;

    #[allow(clippy::identity_op)]
    let out_a = (exp_shared << 26) | (ym << 13) | (xm << 0);
    #[allow(clippy::identity_op)]
    let out_b = (zsign << 15) | (ysign << 14) | (xsign << 13) | (zm << 0);

    (out_a, out_b)
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
fn bitfield_extract_u16(value: u16, offset: u16, bits: u16) -> u16 {
    let mask = (1u16 << bits) - 1u16;
    (value >> offset) & mask
}

#[inline]
pub fn xyz13e6_to_vec3(v: (u32, u16)) -> [f32; 3] {
    let exponent = bitfield_extract(v.0, 26, XYZ13E6_EXPONENT_BITS as u32) as i32
        - XYZ13E6_EXP_BIAS
        - XYZ13E6_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    let xb = bitfield_extract(v.0, 0, XYZ13E6_MANTISSA_BITSU);
    let yb = bitfield_extract(v.0, 13, XYZ13E6_MANTISSA_BITSU);
    let zb = bitfield_extract_u16(v.1, 0, XYZ13E6_MANTISSA_BITSU as u16);

    // Extract the sign bits
    let xs = bitfield_extract_u16(v.1, 13, 1) as u32;
    let ys = bitfield_extract_u16(v.1, 14, 1) as u32;
    let zs = bitfield_extract_u16(v.1, 15, 1) as u32;

    // then xs << 31 shifts it over to the corresponding IEEE 754 sign location
    [
        f32::from_bits((xb as f32).to_bits() | xs << 31) * scale,
        f32::from_bits((yb as f32).to_bits() | ys << 31) * scale,
        f32::from_bits((zb as f32).to_bits() | zs << 31) * scale,
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
            (0.01, 1.65e-6),
            (0.1, 1.32e-5),
            (1.0, 1.82e-4),
            (10.0, 1.69e-3),
            (100.0, 1.36e-2),
            (1000.0, 1.09e-1),
        ] {
            let r = Report::new(dist, DEFUALT_ITERATIONS, true, |v| {
                xyz13e6_to_vec3(vec3_to_xyz13e6(v.into())).into()
            });
            assert!(r.max_dist < max);
        }
    }

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::NEG_ONE,
            xyz13e6_to_vec3(vec3_to_xyz13e6((Vec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::ONE,
            xyz13e6_to_vec3(vec3_to_xyz13e6((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ13E6),
            xyz13e6_to_vec3(vec3_to_xyz13e6(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ13E6),
            xyz13e6_to_vec3(vec3_to_xyz13e6((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ13E6),
            xyz13e6_to_vec3(vec3_to_xyz13e6(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ13E6),
            xyz13e6_to_vec3(vec3_to_xyz13e6((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz13e6_to_vec3(vec3_to_xyz13e6((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz13e6_to_vec3(vec3_to_xyz13e6((Vec3::NAN).into())).into()
        );
    }
}
