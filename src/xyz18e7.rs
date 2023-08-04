use crate::nan_to_zero;

pub const XYZ18E7_EXPONENT_BITS: i32 = 7;
pub const XYZ18E7_MANTISSA_BITS: i32 = 18;
pub const XYZ18E7_MANTISSA_BITSU: u32 = 18;
pub const XYZ18E7_EXP_BIAS: i32 = 63;
pub const XYZ18E7_MAX_VALID_BIASED_EXP: i32 = 127;

/*
pub const MAX_XYZ18E7_EXP: i32 = 64;
pub const XYZ18E7_MANTISSA_VALUES: i32 = 262144;
pub const MAX_XYZ18E7_MANTISSA: i32 = 262143;
pub const MAX_XYZ18E7_MANTISSAU: u32 = 262143;
pub const MAX_XYZ18E7: f32 = 1.8446674e19;
pub const EPSILON_XYZ18E7: f32 = 4.135903e-25;
*/

pub const MAX_XYZ18E7_EXP: u64 = XYZ18E7_MAX_VALID_BIASED_EXP as u64 - XYZ18E7_EXP_BIAS as u64;
pub const XYZ18E7_MANTISSA_VALUES: i32 = 1 << XYZ18E7_MANTISSA_BITS;
pub const MAX_XYZ18E7_MANTISSA: i32 = XYZ18E7_MANTISSA_VALUES - 1;
pub const MAX_XYZ18E7_MANTISSAU: u32 = (XYZ18E7_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ18E7: f32 = (MAX_XYZ18E7_MANTISSA as f32) / XYZ18E7_MANTISSA_VALUES as f32
    * (1u128 << MAX_XYZ18E7_EXP) as f32;
pub const EPSILON_XYZ18E7: f32 =
    (1.0 / XYZ18E7_MANTISSA_VALUES as f32) / (1u64 << XYZ18E7_EXP_BIAS) as f32;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz18e7(xyz: [f32; 3]) -> (u32, u32) {
    let xsign = xyz[0].is_sign_negative() as u32;
    let ysign = xyz[1].is_sign_negative() as u32;
    let zsign = xyz[2].is_sign_negative() as u32;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ18E7);
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ18E7);
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ18E7);

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ18E7_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ18E7_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ18E7_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ18E7_EXP_BIAS - XYZ18E7_MANTISSA_BITS) as f32).exp2();

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
pub fn xyz18e7_to_vec3(v: (u32, u32)) -> [f32; 3] {
    let exponent = bitfield_extract(v.1, 25, XYZ18E7_EXPONENT_BITS as u32) as i32
        - XYZ18E7_EXP_BIAS
        - XYZ18E7_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    let xb = bitfield_extract(v.0, 0, XYZ18E7_MANTISSA_BITSU);
    let yb = bitfield_extract(v.0, 18, XYZ18E7_MANTISSA_BITSU) | bitfield_extract(v.1, 0, 4) << 14;
    let zb = bitfield_extract(v.1, 4, XYZ18E7_MANTISSA_BITSU);

    // Extract the sign bits
    let xs = bitfield_extract(v.1, 22, 1);
    let ys = bitfield_extract(v.1, 23, 1);
    let zs = bitfield_extract(v.1, 24, 1);

    // Then xs << 31 shifts it over to the corresponding IEEE 754 sign location
    [
        f32::from_bits((xb as f32).to_bits() | xs << 31) * scale,
        f32::from_bits((yb as f32).to_bits() | ys << 31) * scale,
        f32::from_bits((zb as f32).to_bits() | zs << 31) * scale,
    ]
}

#[cfg(test)]
pub mod tests {

    use glam::Vec3;

    use crate::{
        test_util::{test_conversion, DEFUALT_ITERATIONS},
        POWLUT,
    };

    use super::*;

    #[test]
    fn get_data_for_plot() {
        dbg!(
            MAX_XYZ18E7_EXP,
            XYZ18E7_MANTISSA_VALUES,
            MAX_XYZ18E7_MANTISSA,
            MAX_XYZ18E7_MANTISSAU,
            MAX_XYZ18E7,
            EPSILON_XYZ18E7,
        );
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..65 {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                xyz18e7_to_vec3(vec3_to_xyz18e7([v.x, v.y, v.z])).into()
            });
            println!("{:.8}\t{:.8}\t{:.8}", n, max, avg);
        }
    }

    pub fn print_typ_ranges(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT[i];
            if n > MAX_XYZ18E7 {
                break;
            }
            let (max, _avg) = test_conversion(n, iterations, false, false, |v| {
                xyz18e7_to_vec3(vec3_to_xyz18e7([v.x, v.y, v.z])).into()
            });
            print!(" {:.8} |", max);
        }
        println!("");
    }

    #[test]
    pub fn print_table_row() {
        print!("| xyz18e7 | 8 | {} | true | ", MAX_XYZ18E7);
    }

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::NEG_ONE,
            xyz18e7_to_vec3(vec3_to_xyz18e7((Vec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::ONE,
            xyz18e7_to_vec3(vec3_to_xyz18e7((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ18E7),
            xyz18e7_to_vec3(vec3_to_xyz18e7((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz18e7_to_vec3(vec3_to_xyz18e7((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz18e7_to_vec3(vec3_to_xyz18e7((Vec3::NAN).into())).into()
        );
    }
}
