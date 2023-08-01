use crate::nan_to_zero64;

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
    let xsign = xyz[0].is_sign_positive() as u32;
    let ysign = xyz[1].is_sign_positive() as u32;
    let zsign = xyz[2].is_sign_positive() as u32;

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

    let xsign = bitfield_extract(v.1, 22, 1) as f64 * 2.0 - 1.0;
    let ysign = bitfield_extract(v.1, 23, 1) as f64 * 2.0 - 1.0;
    let zsign = bitfield_extract(v.1, 24, 1) as f64 * 2.0 - 1.0;

    let xm = bitfield_extract(v.0, 0, XYZ18E7_MANTISSA_BITSU);
    let ym = bitfield_extract(v.0, 18, XYZ18E7_MANTISSA_BITSU) | bitfield_extract(v.1, 0, 4) << 14;
    let zm = bitfield_extract(v.1, 4, XYZ18E7_MANTISSA_BITSU);

    [
        xsign * xm as f64 * scale,
        ysign * ym as f64 * scale,
        zsign * zm as f64 * scale,
    ]
}

#[cfg(test)]
pub mod tests {

    use glam::DVec3;
    use rand::Rng;

    use crate::{test_util::DEFUALT_ITERATIONS, POWLUT64};

    use super::*;

    pub fn test_conversion<F>(
        distance: f64,
        iterations: usize,
        print: bool,
        unsigned: bool,
        proc: F,
    ) -> (f64, f64)
    where
        F: Fn(DVec3) -> DVec3,
    {
        let mut rng = rand::thread_rng();

        let mut max_relative_error = 0.0;
        let mut max_error_orig = DVec3::ZERO;
        let mut max_error_xyz_decoded = DVec3::ZERO;

        let mut max_abs_error = 0.0;
        let mut max_abs_error_orig = DVec3::ZERO;
        let mut max_abs_error_decoded = DVec3::ZERO;

        let mut max_nor_dist = 0.0;
        let mut max_nor_dist_orig = DVec3::ZERO;
        let mut max_nor_dist_decoded = DVec3::ZERO;

        let mut max_dist = 0.0;
        let mut max_dist_orig = DVec3::ZERO;
        let mut max_dist_decoded = DVec3::ZERO;

        let min = if unsigned { 0.0 } else { -distance };
        let max = distance;

        let mut avg_dist = 0.0;
        for _ in 0..iterations {
            let orig = DVec3::from([
                rng.gen_range(min..max),
                rng.gen_range(min..max),
                rng.gen_range(min..max),
            ]);

            let decoded = proc(orig);

            for i in 0..3 {
                let abs_diff = (orig[i] - decoded[i]).abs();
                avg_dist += abs_diff;
                let relative_error = if orig[i] != 0.0 {
                    abs_diff / orig[i]
                } else {
                    abs_diff
                };

                if relative_error > max_relative_error {
                    max_relative_error = relative_error;
                    max_error_orig = orig;
                    max_error_xyz_decoded = decoded;
                }

                if abs_diff > max_abs_error {
                    max_abs_error = abs_diff;
                    max_abs_error_orig = orig;
                    max_abs_error_decoded = decoded;
                }

                let a = DVec3::from(orig);
                let b = DVec3::from(decoded);
                let nor_dist = a.normalize_or_zero().distance(b.normalize_or_zero());
                if b.normalize_or_zero().length() != 0.0 {
                    if nor_dist > max_nor_dist {
                        max_nor_dist = nor_dist;
                        max_nor_dist_orig = orig;
                        max_nor_dist_decoded = decoded;
                    }
                }

                let dist = a.distance(b);
                if dist > max_dist {
                    max_dist = dist;
                    max_dist_orig = orig;
                    max_dist_decoded = decoded;
                }
            }
        }
        avg_dist /= iterations as f64;
        if print {
            println!("\nMaximum Relative Error:");
            println!("Error:\t {}", max_relative_error);
            println!("Original:\t {:?}", max_error_orig);
            println!("Decoded:\t {:?}", max_error_xyz_decoded);

            println!("\nMaximum Absolute Error:");
            println!("Error:\t {}", max_abs_error);
            println!("Original:\t {:?}", max_abs_error_orig);
            println!("Decoded:\t {:?}", max_abs_error_decoded);

            println!("\nMaximum Normalized Distance:");
            println!("Distance: {}", max_nor_dist);
            println!("Original: {:?}", max_nor_dist_orig);
            println!("Decoded: {:?}", max_nor_dist_decoded);

            println!("\nAt Maximum Distance:");
            println!("Original:\t {:?}", max_dist_orig);
            println!("Decoded:\t {:?}", max_dist_decoded);

            println!("Max Dist:\t {}", max_dist);
            println!("Avg Dist:\t {:?}", avg_dist);
        }
        (max_dist, avg_dist)
    }

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
            let mut n = i as f64 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                xyz18e7_to_vec3(vec3_to_xyz18e7([v.x, v.y, v.z])).into()
            });
            println!("{:.10}\t{:.10}\t{:.10}", n, max, avg);
        }
    }

    #[test]
    fn get_data_for_plot_f32() {
        dbg!(f32::MAX);
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..65 {
            let mut n = i as f64 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                DVec3::new(v.x as f32 as f64, v.y as f32 as f64, v.z as f32 as f64)
            });
            println!("{:.10}\t{:.10}\t{:.10}", n, max, avg);
        }
    }

    pub fn print_typ_ranges(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT64[i];
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

    pub fn print_table_row() {
        print!("| xyz18e7 | 8 | {} | true | ", MAX_XYZ18E7);
    }

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
