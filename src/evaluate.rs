#[cfg(test)]
pub mod test_util {
    use glam::Vec3;
    use rand::Rng;

    use super::*;
    pub const DEFUALT_ITERATIONS: usize = 1000000;

    #[derive(Default)]
    pub struct Report {
        pub max_relative_error: f32,
        pub max_error_orig: Vec3,
        pub max_error_xyz_decoded: Vec3,

        pub max_abs_error: f32,
        pub max_abs_error_orig: Vec3,
        pub max_abs_error_decoded: Vec3,

        pub max_nor_dist: f32,
        pub max_nor_dist_orig: Vec3,
        pub max_nor_dist_decoded: Vec3,

        pub max_dist: f32,
        pub max_dist_orig: Vec3,
        pub max_dist_decoded: Vec3,

        pub avg_dist: f32,
    }

    impl Report {
        pub fn new<F>(distance: f32, iterations: usize, signed: bool, proc: F) -> Report
        where
            F: Fn(Vec3) -> Vec3,
        {
            let mut report = Report::default();
            let mut rng = rand::thread_rng();

            let min = if signed { -distance } else { 0.0 };
            let max = distance;

            for _ in 0..iterations {
                let orig = Vec3::from([
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                ]);

                let decoded = proc(orig);

                for i in 0..3 {
                    let abs_diff = (orig[i] - decoded[i]).abs();
                    report.avg_dist += abs_diff;
                    let relative_error = if orig[i] != 0.0 {
                        abs_diff / orig[i]
                    } else {
                        abs_diff
                    };

                    if relative_error > report.max_relative_error {
                        report.max_relative_error = relative_error;
                        report.max_error_orig = orig;
                        report.max_error_xyz_decoded = decoded;
                    }

                    if abs_diff > report.max_abs_error {
                        report.max_abs_error = abs_diff;
                        report.max_abs_error_orig = orig;
                        report.max_abs_error_decoded = decoded;
                    }

                    let a = Vec3::from(orig);
                    let b = Vec3::from(decoded);
                    let nor_dist = a.normalize_or_zero().distance(b.normalize_or_zero());
                    if b.normalize_or_zero().length() != 0.0 {
                        if nor_dist > report.max_nor_dist {
                            report.max_nor_dist = nor_dist;
                            report.max_nor_dist_orig = orig;
                            report.max_nor_dist_decoded = decoded;
                        }
                    }

                    let dist = a.distance(b);
                    if dist > report.max_dist {
                        report.max_dist = dist;
                        report.max_dist_orig = orig;
                        report.max_dist_decoded = decoded;
                    }
                }
            }
            report.avg_dist /= iterations as f32;
            return report;
        }

        pub fn print(&self) {
            println!("\nMaximum Relative Error:");
            println!("Error:\t {}", self.max_relative_error);
            println!("Original:\t {:?}", self.max_error_orig);
            println!("Decoded:\t {:?}", self.max_error_xyz_decoded);

            println!("\nMaximum Absolute Error:");
            println!("Error:\t {}", self.max_abs_error);
            println!("Original:\t {:?}", self.max_abs_error_orig);
            println!("Decoded:\t {:?}", self.max_abs_error_decoded);

            println!("\nMaximum Normalized Distance:");
            println!("Distance: {}", self.max_nor_dist);
            println!("Original: {:?}", self.max_nor_dist_orig);
            println!("Decoded: {:?}", self.max_nor_dist_decoded);

            println!("\nAt Maximum Distance:");
            println!("Original:\t {:?}", self.max_dist_orig);
            println!("Decoded:\t {:?}", self.max_dist_decoded);

            println!("Max Dist:\t {}", self.max_dist);
            println!("Avg Dist:\t {:?}", self.avg_dist);
        }
    }

    pub fn report_set<F>(n: usize, proc: F) -> Vec<(f32, Report)>
    where
        F: Fn(Vec3) -> Vec3,
    {
        let mut set = Vec::new();
        for i in 1..n {
            let mut n = i as f32 * 0.25;
            n = n.exp2() - 1.0;
            let report = Report::new(n, DEFUALT_ITERATIONS, false, |v| proc(v));
            set.push((n, report));
        }
        set
    }

    pub fn typ_ranges<F>(iterations: usize, max: f32, signed: bool, proc: F) -> Vec<(f32, Report)>
    where
        F: Fn(Vec3) -> Vec3,
    {
        let mut set = Vec::new();
        for i in 0..6 {
            let n = POWLUT[i];
            if n > max {
                break;
            }
            let report = Report::new(n, iterations, signed, |v| proc(v));
            set.push((n, report));
        }
        set
    }
}

// 10f32.powi(n - 3) is not accurate enough
pub const POWLUT: [f32; 10] = [
    0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
];
pub const POWLUT64: [f64; 10] = [
    0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
];

#[cfg(test)]
pub mod print_table {

    use glam::{vec3, Vec3};
    use tabled::{
        settings::{object::Columns, Format, Modify, Style},
        Table, Tabled,
    };

    use crate::{
        evaluate::test_util::{typ_ranges, Report, DEFUALT_ITERATIONS},
        rgb9e5::{rgb9e5_to_vec3, vec3_to_rgb9e5},
        xyz13e6::{vec3_to_xyz13e6, xyz13e6_to_vec3},
        xyz14e3::{vec3_to_xyz14e3, xyz14e3_to_vec3},
        xyz18e7::{vec3_to_xyz18e7, xyz18e7_to_vec3},
        xyz8e5::{vec3_to_xyz8e5, xyz8e5_to_vec3},
        xyz9e2::{vec3_to_xyz9e2, xyz9e2_to_vec3},
    };

    use crate::rgb9e5;
    use crate::xyz13e6;
    use crate::xyz14e3;
    use crate::xyz18e7;
    use crate::xyz8e5;
    use crate::xyz9e2;

    fn f16_roundtrip(v: Vec3) -> Vec3 {
        vec3(
            half::f16::from_f32(v.x).into(),
            half::f16::from_f32(v.y).into(),
            half::f16::from_f32(v.z).into(),
        )
    }

    fn unorm8_roundtrip(v: Vec3) -> Vec3 {
        (v.clamp(Vec3::ZERO, Vec3::ONE) * 255.0 + 0.5).floor() / 255.0
    }

    #[derive(Tabled, Clone)]
    struct TypRangesRow {
        #[tabled(rename = "Name")]
        name: &'static str,
        #[tabled(rename = "Bytes")]
        bytes: u8,
        #[tabled(rename = "Signed")]
        signed: bool,
        #[tabled(rename = "Max Val")]
        max: f32,
        #[tabled(rename = "Epsilon")]
        epsilon: f32,
        #[tabled(rename = "0.01 Max Δ")]
        n01maxd: f32,
        #[tabled(rename = "0.1 Max Δ")]
        np1maxd: f32,
        #[tabled(rename = "1.0 Max Δ")]
        n1maxd: f32,
        #[tabled(rename = "10.0 Max Δ")]
        n10maxd: f32,
        #[tabled(rename = "100 Max Δ")]
        n100maxd: f32,
        #[tabled(rename = "1000 Max Δ")]
        n1000maxd: f32,
    }

    fn row(
        name: &'static str,
        bytes: u8,
        max: f32,
        epsilon: f32,
        signed: bool,
        typ: Vec<(f32, Report)>,
    ) -> TypRangesRow {
        TypRangesRow {
            name,
            bytes,
            max,
            epsilon,
            signed,
            n01maxd: typ
                .get(0)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
            np1maxd: typ
                .get(1)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
            n1maxd: typ
                .get(2)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
            n10maxd: typ
                .get(3)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
            n100maxd: typ
                .get(4)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
            n1000maxd: typ
                .get(5)
                .and_then(|r| Some(r.1.max_dist))
                .unwrap_or(f32::INFINITY),
        }
    }

    pub fn print_full_table() {
        let iters = DEFUALT_ITERATIONS * 100;

        let mut table = Vec::new();
        table.push(row(
            xyz8e5::NAME,
            xyz8e5::BYTES,
            xyz8e5::MAX_XYZ8E5,
            xyz8e5::EPSILON_XYZ8E5,
            xyz8e5::SIGNED,
            typ_ranges(iters, xyz8e5::MAX_XYZ8E5, xyz8e5::SIGNED, |v| {
                xyz8e5_to_vec3(vec3_to_xyz8e5(v.into())).into()
            }),
        ));

        table.push(row(
            rgb9e5::NAME,
            rgb9e5::BYTES,
            rgb9e5::MAX_RGB9E5,
            rgb9e5::EPSILON_RGB9E5,
            rgb9e5::SIGNED,
            typ_ranges(iters, rgb9e5::MAX_RGB9E5, rgb9e5::SIGNED, |v| {
                rgb9e5_to_vec3(vec3_to_rgb9e5(v.into())).into()
            }),
        ));

        table.push(row(
            xyz9e2::NAME,
            xyz9e2::BYTES,
            xyz9e2::MAX_XYZ9E2,
            xyz9e2::EPSILON_XYZ9E2,
            xyz9e2::SIGNED,
            typ_ranges(iters, xyz9e2::MAX_XYZ9E2, xyz9e2::SIGNED, |v| {
                xyz9e2_to_vec3(vec3_to_xyz9e2(v.into())).into()
            }),
        ));

        table.push(row(
            "3x f16",
            6,
            half::f16::MAX.into(),
            half::f16::EPSILON.into(),
            true,
            typ_ranges(iters, half::f16::MAX.into(), true, |v| f16_roundtrip(v)),
        ));

        table.push(row(
            xyz13e6::NAME,
            xyz13e6::BYTES,
            xyz13e6::MAX_XYZ13E6,
            xyz13e6::EPSILON_XYZ13E6,
            xyz13e6::SIGNED,
            typ_ranges(iters, xyz13e6::MAX_XYZ13E6, xyz13e6::SIGNED, |v| {
                xyz13e6_to_vec3(vec3_to_xyz13e6(v.into())).into()
            }),
        ));

        table.push(row(
            xyz14e3::NAME,
            xyz14e3::BYTES,
            xyz14e3::MAX_XYZ14E3,
            xyz14e3::EPSILON_XYZ14E3,
            xyz14e3::SIGNED,
            typ_ranges(iters, xyz14e3::MAX_XYZ14E3, xyz14e3::SIGNED, |v| {
                xyz14e3_to_vec3(vec3_to_xyz14e3(v.into())).into()
            }),
        ));

        table.push(row(
            xyz18e7::NAME,
            xyz18e7::BYTES,
            xyz18e7::MAX_XYZ18E7,
            xyz18e7::EPSILON_XYZ18E7,
            xyz18e7::SIGNED,
            typ_ranges(iters, xyz18e7::MAX_XYZ18E7, xyz18e7::SIGNED, |v| {
                xyz18e7_to_vec3(vec3_to_xyz18e7(v.into())).into()
            }),
        ));

        table.push(row(
            "3x 8unorm",
            3,
            1.0,
            0.00392156862745098,
            false,
            typ_ranges(iters, 1.0, false, |v| unorm8_roundtrip(v)),
        ));

        println!(
            "{}",
            Table::new(table)
                .with(
                    Modify::new(Columns::new(5..)).with(Format::positioned(|s, p| if s == "inf" {
                        String::new()
                    } else if p.0 == 0 {
                        format!("{}", s)
                    } else {
                        let v = s.parse::<f32>().unwrap();
                        format!("{:.2e}", v).replace("e0", "")
                    }))
                )
                .with(
                    Modify::new(Columns::new(3..=3)).with(Format::positioned(|s, p| {
                        if s.len() > 5 && p.0 > 0 {
                            let v = s.parse::<f32>().unwrap();
                            format!("{:.2e}", v)
                        } else {
                            format!("{}", s)
                        }
                    }))
                )
                .with(
                    Modify::new(Columns::new(4..=4)).with(Format::positioned(|s, p| {
                        if p.0 > 0 {
                            let v = s.parse::<f32>().unwrap();
                            format!("{:.2e}", v)
                        } else {
                            format!("{}", s)
                        }
                    }))
                )
                .with(Style::markdown())
                .to_string()
        );
    }
}
