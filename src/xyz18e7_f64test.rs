use crate::nan_to_zero64;

pub const NAME: &str = "xyz18e7";
pub const BYTES: u8 = 8;
pub const SIGNED: bool = true;

pub const XYZ18E7_EXPONENT_BITS: i32 = 7;
pub const XYZ18E7_MANTISSA_BITS: i32 = 18;
pub const XYZ18E7_MANTISSA_BITSU: u32 = 18;
pub const XYZ18E7_EXP_BIAS: i32 = 63;
pub const XYZ18E7_MAX_VALID_BIASED_EXP: i32 = 127;

/*
pub const MAX_XYZ18E7_EXP: i32 = 32;
pub const XYZ18E7_MANTISSA_VALUES: i32 = 8192;
pub const MAX_XYZ18E7_MANTISSA: i32 = 8191;
pub const MAX_XYZ18E7_MANTISSAU: u32 = 8191;
pub const MAX_XYZ18E7: f64 = 4294443000.0;
pub const EPSILON_XYZ18E7: f64 = -5.684342e-14;
*/

pub const MAX_XYZ18E7_EXP: u64 = XYZ18E7_MAX_VALID_BIASED_EXP as u64 - XYZ18E7_EXP_BIAS as u64;
pub const XYZ18E7_MANTISSA_VALUES: i32 = 1 << XYZ18E7_MANTISSA_BITS;
pub const MAX_XYZ18E7_MANTISSA: i32 = XYZ18E7_MANTISSA_VALUES - 1;
pub const MAX_XYZ18E7_MANTISSAU: u32 = (XYZ18E7_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ18E7: f64 = (MAX_XYZ18E7_MANTISSA as f64) / XYZ18E7_MANTISSA_VALUES as f64
    * (1u128 << MAX_XYZ18E7_EXP) as f64;
pub const EPSILON_XYZ18E7: f64 =
    (1.0 / XYZ18E7_MANTISSA_VALUES as f64) / (1u64 << XYZ18E7_EXP_BIAS) as f64;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz18e7(xyz: [f64; 3]) -> (u32, u32) {
    let xsign = xyz[0].is_sign_negative() as u32;
    let ysign = xyz[1].is_sign_negative() as u32;
    let zsign = xyz[2].is_sign_negative() as u32;

    let xc = nan_to_zero64(xyz[0].abs()).min(MAX_XYZ18E7);
    let yc = nan_to_zero64(xyz[1].abs()).min(MAX_XYZ18E7);
    let zc = nan_to_zero64(xyz[2].abs()).min(MAX_XYZ18E7);

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ18E7_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ18E7_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ18E7_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ18E7_EXP_BIAS - XYZ18E7_MANTISSA_BITS) as f64).exp2();

    let maxm = (maxxyz / denom + 0.5).floor() as i32;
    if maxm == MAX_XYZ18E7_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= XYZ18E7_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_XYZ18E7_MANTISSA);
    }

    let xm = (xc / denom + 0.5).floor() as i32;
    let ym = (yc / denom + 0.5).floor() as i32;
    let zm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(xm <= MAX_XYZ18E7_MANTISSA);
    debug_assert!(ym <= MAX_XYZ18E7_MANTISSA);
    debug_assert!(zm <= MAX_XYZ18E7_MANTISSA);
    debug_assert!(xm >= 0);
    debug_assert!(ym >= 0);
    debug_assert!(zm >= 0);

    debug_assert_eq!(xm as u32, xm as u32 & MAX_XYZ18E7_MANTISSAU);
    debug_assert_eq!(ym as u32, ym as u32 & MAX_XYZ18E7_MANTISSAU);
    debug_assert_eq!(zm as u32, zm as u32 & MAX_XYZ18E7_MANTISSAU);

    let exp_shared = exp_shared as u32;

    let xm = xm as u32;
    let ym = ym as u32;
    let zm = zm as u32;

    #[allow(clippy::identity_op)]
    let out_a = (ym << 18) | (xm << 0);
    #[allow(clippy::identity_op)]
    let out_b =
        (exp_shared << 25) | (zsign << 24) | (ysign << 23) | (xsign << 22) | (zm << 4) | (ym >> 14);

    (out_a, out_b)
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
pub fn xyz18e7_to_vec3(v: (u32, u32)) -> [f64; 3] {
    let exponent = bitfield_extract(v.1, 25, XYZ18E7_EXPONENT_BITS as u32) as i32
        - XYZ18E7_EXP_BIAS
        - XYZ18E7_MANTISSA_BITS;
    let scale = (exponent as f64).exp2();

    let xsign = (bitfield_extract(v.1, 22, 1) << 1) as f64 - 1.0;
    let ysign = (bitfield_extract(v.1, 23, 1) << 1) as f64 - 1.0;
    let zsign = (bitfield_extract(v.1, 24, 1) << 1) as f64 - 1.0;

    let xm = bitfield_extract(v.0, 0, XYZ18E7_MANTISSA_BITSU);
    let ym = bitfield_extract(v.0, 18, XYZ18E7_MANTISSA_BITSU) | bitfield_extract(v.1, 0, 4) << 14;
    let zm = bitfield_extract(v.1, 4, XYZ18E7_MANTISSA_BITSU);

    [
        -xsign * xm as f64 * scale,
        -ysign * ym as f64 * scale,
        -zsign * zm as f64 * scale,
    ]
}

#[cfg(test)]
pub mod tests {

    use glam::DVec3;

    use super::*;

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            DVec3::NEG_ONE,
            xyz18e7_to_vec3(vec3_to_xyz18e7((DVec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            DVec3::ONE,
            xyz18e7_to_vec3(vec3_to_xyz18e7((DVec3::ONE).into())).into()
        );
        debug_assert_eq!(
            DVec3::splat(MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7(DVec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            DVec3::splat(-MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7((-DVec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            DVec3::splat(MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7(DVec3::MAX.into())).into()
        );
        debug_assert_eq!(
            DVec3::splat(-MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7((-DVec3::MAX).into())).into()
        );
        debug_assert_eq!(
            DVec3::ZERO,
            xyz18e7_to_vec3(vec3_to_xyz18e7((DVec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            DVec3::ZERO,
            xyz18e7_to_vec3(vec3_to_xyz18e7((DVec3::NAN).into())).into()
        );
    }
}
