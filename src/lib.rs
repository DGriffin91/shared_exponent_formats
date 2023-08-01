pub mod rgb9e5;
pub mod xyz13e6;
pub mod xyz14e3;
pub mod xyz8e5;
pub mod xyz9e2;

pub fn nan_to_zero(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

// 10f32.powi(n - 3) is not accurate enough
pub const POWLUT: [f32; 10] = [
    0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
];

#[cfg(test)]
pub mod test_util {
    use glam::{vec3, Vec3};
    use rand::Rng;

    use crate::POWLUT;

    pub const DEFUALT_ITERATIONS: usize = 1000000;

    pub fn test_conversion<F>(
        distance: f32,
        iterations: usize,
        print: bool,
        unsigned: bool,
        proc: F,
    ) -> (f32, f32)
    where
        F: Fn(Vec3) -> Vec3,
    {
        let mut rng = rand::thread_rng();

        let mut max_relative_error = 0.0;
        let mut max_error_orig = Vec3::ZERO;
        let mut max_error_xyz_decoded = Vec3::ZERO;

        let mut max_abs_error = 0.0;
        let mut max_abs_error_orig = Vec3::ZERO;
        let mut max_abs_error_decoded = Vec3::ZERO;

        let mut max_nor_dist = 0.0;
        let mut max_nor_dist_orig = Vec3::ZERO;
        let mut max_nor_dist_decoded = Vec3::ZERO;

        let mut max_dist = 0.0;
        let mut max_dist_orig = Vec3::ZERO;
        let mut max_dist_decoded = Vec3::ZERO;

        let min = if unsigned { 0.0 } else { -distance };
        let max = distance;

        let mut avg_dist = 0.0;
        for _ in 0..iterations {
            let orig = Vec3::from([
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

                let a = Vec3::from(orig);
                let b = Vec3::from(decoded);
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
        avg_dist /= iterations as f32;
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
    fn get_data_for_plot_f16() {
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..57 {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, false, |v| {
                vec3(
                    half::f16::from_f32(v.x).into(),
                    half::f16::from_f32(v.y).into(),
                    half::f16::from_f32(v.z).into(),
                )
            });
            println!("{:.8}\t{:.8}\t{:.8}", n, max, avg);
        }
    }

    fn print_typ_ranges_f16(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT[i];
            if n > half::f16::MAX.into() {
                break;
            }
            let (max, _avg) = test_conversion(n, iterations, false, false, |v| {
                vec3(
                    half::f16::from_f32(v.x).into(),
                    half::f16::from_f32(v.y).into(),
                    half::f16::from_f32(v.z).into(),
                )
            });
            print!(" {:.8} |", max);
        }
        println!("");
    }

    #[test]
    pub fn print_table_row_f16() {
        print!("| 3x f16 | 6 | {} | true | ", half::f16::MAX);
    }

    #[test]
    fn get_data_for_plot_8unorm() {
        println!("RANGE   \tMAX      \tAVG");
        for i in 1..5 {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let (max, avg) = test_conversion(n, DEFUALT_ITERATIONS, false, true, |v| {
                (v.clamp(Vec3::ZERO, Vec3::ONE) * 255.0 + 0.5).floor() / 255.0
            });
            println!("{:.8}\t{:.8}\t{:.8}", n, max, avg);
        }
    }

    fn print_typ_ranges_8unorm(iterations: usize) {
        for i in 0..6 {
            let n = POWLUT[i];
            if n > 1.1 {
                break;
            }
            let (max, _avg) = test_conversion(n, iterations, false, true, |v| {
                (v.clamp(Vec3::ZERO, Vec3::ONE) * 255.0 + 0.5).floor() / 255.0
            });
            print!(" {:.8} |", max);
        }
        println!("");
    }

    #[test]
    pub fn print_table_row_8unorm() {
        print!("| 8unorm | 3 | 1 | false | ");
    }

    #[test]
    pub fn print_full_table() {
        let iterations = DEFUALT_ITERATIONS * 10;
        println!("| Name     | Bytes | Max value  | Signed |  0.01 Max Δ | 0.1 Max Δ  | 1.0 Max Δ  | 10.0 Max Δ | 1000 Max Δ | 10000 Max Δ |");
        println!("| -------- | ----- | ---------- | ------ | ----------- | ---------- | ---------- | ---------- | ---------- | ----------- |");
        crate::xyz8e5::tests::print_table_row();
        crate::xyz8e5::tests::print_typ_ranges(iterations);
        crate::rgb9e5::tests::print_table_row();
        crate::rgb9e5::tests::print_typ_ranges(iterations);
        crate::xyz9e2::tests::print_table_row();
        crate::xyz9e2::tests::print_typ_ranges(iterations);
        print_table_row_f16();
        print_typ_ranges_f16(iterations);
        crate::xyz13e6::tests::print_table_row();
        crate::xyz13e6::tests::print_typ_ranges(iterations);
        crate::xyz14e3::tests::print_table_row();
        crate::xyz14e3::tests::print_typ_ranges(iterations);
        print_table_row_8unorm();
        print_typ_ranges_8unorm(iterations);
    }
}
