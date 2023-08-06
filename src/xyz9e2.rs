use crate::nan_to_zero;

pub const NAME: &str = "xyz9e2";
pub const BYTES: u8 = 4;
pub const SIGNED: bool = true;

pub const XYZ9E2_EXPONENT_BITS: i32 = 2;
pub const XYZ9E2_MANTISSA_BITS: i32 = 9;
pub const XYZ9E2_MANTISSA_BITSU: u32 = 9;
pub const XYZ9E2_EXP_BIAS: i32 = 3;
pub const XYZ9E2_MAX_VALID_BIASED_EXP: i32 = 3;

// MAX_XYZ9E2 would be 0.9980469, this is used to scale to exactly 1.0
pub const NORM_MULT: f32 = 1.0019569;

/*
pub const MAX_XYZ9E2_EXP: i32 = 0;
pub const XYZ9E2_MANTISSA_VALUES: i32 = 512;
pub const MAX_XYZ9E2_MANTISSA: i32 = 511;
pub const MAX_XYZ9E2_MANTISSAU: u32 = 511;
pub const MAX_XYZ9E2: f32 = 1.0;
pub const EPSILON_XYZ9E2: f32 = 0.00024414063;
*/

pub const MAX_XYZ9E2_EXP: i32 = XYZ9E2_MAX_VALID_BIASED_EXP - XYZ9E2_EXP_BIAS;
pub const XYZ9E2_MANTISSA_VALUES: i32 = 1 << XYZ9E2_MANTISSA_BITS;
pub const MAX_XYZ9E2_MANTISSA: i32 = XYZ9E2_MANTISSA_VALUES - 1;
pub const MAX_XYZ9E2_MANTISSAU: u32 = (XYZ9E2_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ9E2: f32 = (MAX_XYZ9E2_MANTISSA as f32) / XYZ9E2_MANTISSA_VALUES as f32
    * (1 << MAX_XYZ9E2_EXP) as f32
    * NORM_MULT;
pub const EPSILON_XYZ9E2: f32 =
    (1.0 / XYZ9E2_MANTISSA_VALUES as f32) / (1 << XYZ9E2_EXP_BIAS) as f32;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz9e2(xyz: [f32; 3]) -> u32 {
    let xsign = xyz[0].is_sign_negative() as u32;
    let ysign = xyz[1].is_sign_negative() as u32;
    let zsign = xyz[2].is_sign_negative() as u32;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ9E2) / NORM_MULT;
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ9E2) / NORM_MULT;
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ9E2) / NORM_MULT;

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ9E2_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ9E2_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ9E2_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ9E2_EXP_BIAS - XYZ9E2_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxxyz / denom + 0.5).floor() as i32;
    if maxm == MAX_XYZ9E2_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= XYZ9E2_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_XYZ9E2_MANTISSA);
    }

    let xm = (xc / denom + 0.5).floor() as i32;
    let ym = (yc / denom + 0.5).floor() as i32;
    let zm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(xm <= MAX_XYZ9E2_MANTISSA);
    debug_assert!(ym <= MAX_XYZ9E2_MANTISSA);
    debug_assert!(zm <= MAX_XYZ9E2_MANTISSA);
    debug_assert!(xm >= 0);
    debug_assert!(ym >= 0);
    debug_assert!(zm >= 0);

    debug_assert_eq!(xm as u32, xm as u32 & MAX_XYZ9E2_MANTISSAU);
    debug_assert_eq!(ym as u32, ym as u32 & MAX_XYZ9E2_MANTISSAU);
    debug_assert_eq!(zm as u32, zm as u32 & MAX_XYZ9E2_MANTISSAU);

    let xm = xm as u32 | xsign << 9;
    let ym = ym as u32 | ysign << 9;
    let zm = zm as u32 | zsign << 9;
    let exp_shared = exp_shared as u32;

    #[allow(clippy::identity_op)]
    let ret = (exp_shared << 30) | (zm << 20) | (ym << 10) | (xm << 0);

    ret
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
pub fn xyz9e2_to_vec3(v: u32) -> [f32; 3] {
    let exponent = bitfield_extract(v, 30, XYZ9E2_EXPONENT_BITS as u32) as i32
        - XYZ9E2_EXP_BIAS
        - XYZ9E2_MANTISSA_BITS;
    let scale = (exponent as f32).exp2() * NORM_MULT;

    // Extract both the mantissa and sign at the same time.
    let xb = bitfield_extract(v, 0, XYZ9E2_MANTISSA_BITSU + 1);
    let yb = bitfield_extract(v, 10, XYZ9E2_MANTISSA_BITSU + 1);
    let zb = bitfield_extract(v, 20, XYZ9E2_MANTISSA_BITSU + 1);

    // xb & 0x1FFu masks out for just the mantissa
    let xm = ((xb & 0x1FFu32) as f32).to_bits();
    let ym = ((yb & 0x1FFu32) as f32).to_bits();
    let zm = ((zb & 0x1FFu32) as f32).to_bits();

    // xb & 0x200u << 23u masks out just the sign bit and shifts it over
    // to the corresponding IEEE 754 sign location
    [
        f32::from_bits(xm | (xb & 0x200u32) << 22u32) * scale,
        f32::from_bits(ym | (yb & 0x200u32) << 22u32) * scale,
        f32::from_bits(zm | (zb & 0x200u32) << 22u32) * scale,
    ]
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use glam::Vec3;

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::NEG_ONE,
            xyz9e2_to_vec3(vec3_to_xyz9e2((Vec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::ONE,
            xyz9e2_to_vec3(vec3_to_xyz9e2((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ9E2),
            xyz9e2_to_vec3(vec3_to_xyz9e2(Vec3::splat(-1.0).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ9E2),
            xyz9e2_to_vec3(vec3_to_xyz9e2(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ9E2),
            xyz9e2_to_vec3(vec3_to_xyz9e2((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ9E2),
            xyz9e2_to_vec3(vec3_to_xyz9e2(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ9E2),
            xyz9e2_to_vec3(vec3_to_xyz9e2((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz9e2_to_vec3(vec3_to_xyz9e2((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz9e2_to_vec3(vec3_to_xyz9e2((Vec3::NAN).into())).into()
        );
    }
}
