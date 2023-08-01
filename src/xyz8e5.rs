use crate::nan_to_zero;

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
    let rsign = xyz[0].is_sign_positive() as u32;
    let gsign = xyz[1].is_sign_positive() as u32;
    let bsign = xyz[2].is_sign_positive() as u32;

    let xc = nan_to_zero(xyz[0].abs()).min(MAX_XYZ8E5);
    let yc = nan_to_zero(xyz[1].abs()).min(MAX_XYZ8E5);
    let zc = nan_to_zero(xyz[2].abs()).min(MAX_XYZ8E5);

    let maxxyz = xc.max(yc).max(zc);
    let mut exp_shared =
        (-XYZ8E5_EXP_BIAS - 1).max(maxxyz.log2().floor() as i32) + 1 + XYZ8E5_EXP_BIAS;

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

    let rm = (xc / denom + 0.5).floor() as i32;
    let gm = (yc / denom + 0.5).floor() as i32;
    let bm = (zc / denom + 0.5).floor() as i32;

    debug_assert!(rm <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(gm <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(bm <= MAX_XYZ8E5_MANTISSA);
    debug_assert!(rm >= 0);
    debug_assert!(gm >= 0);
    debug_assert!(bm >= 0);

    debug_assert_eq!(rm as u32, rm as u32 & MAX_XYZ8E5_MANTISSAU);
    debug_assert_eq!(gm as u32, gm as u32 & MAX_XYZ8E5_MANTISSAU);
    debug_assert_eq!(bm as u32, bm as u32 & MAX_XYZ8E5_MANTISSAU);

    let rm = rm as u32 | rsign << 8;
    let gm = gm as u32 | gsign << 8;
    let bm = bm as u32 | bsign << 8;
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
pub fn xyz8e5_to_vec3(v: u32) -> [f32; 3] {
    let exponent = bitfield_extract(v, 27, XYZ8E5_EXPONENT_BITS as u32) as i32
        - XYZ8E5_EXP_BIAS
        - XYZ8E5_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    let rsign = bitfield_extract(v, 8, 1) as f32 * 2.0 - 1.0;
    let gsign = bitfield_extract(v, 17, 1) as f32 * 2.0 - 1.0;
    let bsign = bitfield_extract(v, 26, 1) as f32 * 2.0 - 1.0;

    [
        rsign * bitfield_extract(v, 0, XYZ8E5_MANTISSA_BITS as u32) as f32 * scale,
        gsign * bitfield_extract(v, 9, XYZ8E5_MANTISSA_BITS as u32) as f32 * scale,
        bsign * bitfield_extract(v, 18, XYZ8E5_MANTISSA_BITS as u32) as f32 * scale,
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
            MAX_XYZ8E5_EXP,
            XYZ8E5_MANTISSA_VALUES,
            MAX_XYZ8E5_MANTISSA,
            MAX_XYZ8E5_MANTISSAU,
            MAX_XYZ8E5,
            EPSILON_XYZ8E5,
        );
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..65 {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                xyz8e5_to_vec3(vec3_to_xyz8e5(v.into())).into()
            });
            println!("{:.8}\t{:.8}\t{:.8}", n, max, avg);
        }
    }

    pub fn print_typ_ranges(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT[i];
            if n > MAX_XYZ8E5 {
                break;
            }
            let (max, _avg) = test_conversion(n, iterations, false, false, |v| {
                xyz8e5_to_vec3(vec3_to_xyz8e5(v.into())).into()
            });
            print!(" {:.8} |", max);
        }
        println!("");
    }

    pub fn print_table_row() {
        print!("| xyz8e5 | 4 | {} | true | ", MAX_XYZ8E5);
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
