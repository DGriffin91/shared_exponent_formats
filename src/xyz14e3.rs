use crate::nan_to_zero;

pub const XYZ14E3_EXPONENT_BITS: i32 = 3;
pub const XYZ14E3_MANTISSA_BITS: i32 = 14;
pub const XYZ14E3_MANTISSA_BITSU: u32 = 14;
pub const XYZ14E3_EXP_BIAS: i32 = 3;
pub const XYZ14E3_MAX_VALID_BIASED_EXP: i32 = 7;

// MAX_XYZ9E2 would be 15.999023, this is used to scale to exactly 16.0
pub const NORM_MULT: f32 = 1.000061;

/*
pub const MAX_XYZ14E3_EXP: u64 = 4;
pub const XYZ14E3_MANTISSA_VALUES: i32 = 16384;
pub const MAX_XYZ14E3_MANTISSA: i32 = 16383;
pub const MAX_XYZ14E3_MANTISSAU: u32 = 16383;
pub const MAX_XYZ14E3: f32 = 16.0;
pub const EPSILON_XYZ14E3: f32 = 7.6293945e-6;
*/

pub const MAX_XYZ14E3_EXP: u64 = XYZ14E3_MAX_VALID_BIASED_EXP as u64 - XYZ14E3_EXP_BIAS as u64;
pub const XYZ14E3_MANTISSA_VALUES: i32 = 1 << XYZ14E3_MANTISSA_BITS;
pub const MAX_XYZ14E3_MANTISSA: i32 = XYZ14E3_MANTISSA_VALUES - 1;
pub const MAX_XYZ14E3_MANTISSAU: u32 = (XYZ14E3_MANTISSA_VALUES - 1) as u32;
pub const MAX_XYZ14E3: f32 = (MAX_XYZ14E3_MANTISSA as f32) / XYZ14E3_MANTISSA_VALUES as f32
    * (1u64 << MAX_XYZ14E3_EXP) as f32
    * NORM_MULT;
pub const EPSILON_XYZ14E3: f32 =
    (1.0 / XYZ14E3_MANTISSA_VALUES as f32) / (1 << XYZ14E3_EXP_BIAS) as f32;

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn vec3_to_xyz14e3(xyz: [f32; 3]) -> (u32, u16) {
    let xsign = xyz[0].is_sign_negative() as u32;
    let ysign = xyz[1].is_sign_negative() as u32;
    let zsign = xyz[2].is_sign_negative() as u32;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ14E3) / NORM_MULT;
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ14E3) / NORM_MULT;
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ14E3) / NORM_MULT;

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ14E3_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ14E3_EXP_BIAS;

    debug_assert!(exp_shared <= XYZ14E3_MAX_VALID_BIASED_EXP);
    debug_assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - XYZ14E3_EXP_BIAS - XYZ14E3_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxxyz / denom + 0.5).floor() as i32;
    if maxm == MAX_XYZ14E3_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        debug_assert!(exp_shared <= XYZ14E3_MAX_VALID_BIASED_EXP);
    } else {
        debug_assert!(maxm <= MAX_XYZ14E3_MANTISSA);
    }

    let xm = (xc / denom + 0.5).floor() as i32;
    let ym = (yc / denom + 0.5).floor() as i32;
    let zm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(xm <= MAX_XYZ14E3_MANTISSA);
    debug_assert!(ym <= MAX_XYZ14E3_MANTISSA);
    debug_assert!(zm <= MAX_XYZ14E3_MANTISSA);
    debug_assert!(xm >= 0);
    debug_assert!(ym >= 0);
    debug_assert!(zm >= 0);

    debug_assert_eq!(xm as u32, xm as u32 & MAX_XYZ14E3_MANTISSAU);
    debug_assert_eq!(ym as u32, ym as u32 & MAX_XYZ14E3_MANTISSAU);
    debug_assert_eq!(zm as u32, zm as u32 & MAX_XYZ14E3_MANTISSAU);

    let exp_shared = exp_shared as u32;

    let xm = xm as u32;
    let ym = ym as u32;
    let zm = zm as u16;

    #[allow(clippy::identity_op)]
    let out_a = ((exp_shared & 1) << 31)
        | (zsign << 30)
        | (ysign << 29)
        | (xsign << 28)
        | (ym << 14)
        | (xm << 0);
    #[allow(clippy::identity_op)]
    let out_b = ((exp_shared & 6) << 13) as u16 | (zm << 0);

    (out_a, out_b)
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1 << bits) - 1;
    (value >> offset) & mask
}

#[inline]
fn bitfield_extract_u16(value: u16, offset: u16, bits: u16) -> u16 {
    let mask = (1 << bits) - 1;
    (value >> offset) & mask
}

#[inline]
pub fn xyz14e3_to_vec3(v: (u32, u16)) -> [f32; 3] {
    let exp = bitfield_extract(v.0, 31, 1) | ((bitfield_extract_u16(v.1, 14, 2) as u32) << 1);
    let exponent = exp as i32 - XYZ14E3_EXP_BIAS - XYZ14E3_MANTISSA_BITS;
    let scale = (exponent as f32).exp2() * NORM_MULT;

    let xb = bitfield_extract(v.0, 0, XYZ14E3_MANTISSA_BITSU);
    let yb = bitfield_extract(v.0, 14, XYZ14E3_MANTISSA_BITSU);
    let zb = bitfield_extract_u16(v.1, 0, XYZ14E3_MANTISSA_BITSU as u16);

    // Extract the sign bits
    let xs = bitfield_extract(v.0, 28, 1) as u32;
    let ys = bitfield_extract(v.0, 29, 1) as u32;
    let zs = bitfield_extract(v.0, 30, 1) as u32;

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
            MAX_XYZ14E3_EXP,
            XYZ14E3_MANTISSA_VALUES,
            MAX_XYZ14E3_MANTISSA,
            MAX_XYZ14E3_MANTISSAU,
            MAX_XYZ14E3,
            EPSILON_XYZ14E3,
        );
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..20 {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                xyz14e3_to_vec3(vec3_to_xyz14e3(v.into())).into()
            });
            println!("{:.8}\t{:.8}\t{:.8}", n, max, avg);
        }
    }

    pub fn print_typ_ranges(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT[i];
            if n > MAX_XYZ14E3 {
                break;
            }
            let (max, _avg) = test_conversion(n, iterations, false, false, |v| {
                xyz14e3_to_vec3(vec3_to_xyz14e3(v.into())).into()
            });
            print!(" {:.8} |", max);
        }
        println!("");
    }

    pub fn print_table_row() {
        print!("| xyz14e3 | 6 | {} | true | ", MAX_XYZ14E3);
    }

    #[test]
    fn test_edge_cases() {
        debug_assert_eq!(
            Vec3::NEG_ONE,
            xyz14e3_to_vec3(vec3_to_xyz14e3((Vec3::NEG_ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::ONE,
            xyz14e3_to_vec3(vec3_to_xyz14e3((Vec3::ONE).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ14E3),
            xyz14e3_to_vec3(vec3_to_xyz14e3(Vec3::INFINITY.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ14E3),
            xyz14e3_to_vec3(vec3_to_xyz14e3((-Vec3::INFINITY).into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(MAX_XYZ14E3),
            xyz14e3_to_vec3(vec3_to_xyz14e3(Vec3::MAX.into())).into()
        );
        debug_assert_eq!(
            Vec3::splat(-MAX_XYZ14E3),
            xyz14e3_to_vec3(vec3_to_xyz14e3((-Vec3::MAX).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz14e3_to_vec3(vec3_to_xyz14e3((Vec3::ZERO).into())).into()
        );
        debug_assert_eq!(
            Vec3::ZERO,
            xyz14e3_to_vec3(vec3_to_xyz14e3((Vec3::NAN).into())).into()
        );
    }
}
