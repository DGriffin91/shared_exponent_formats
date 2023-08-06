#[cfg(test)]
pub mod test_util {
    use glam::DVec3;
    use rand::Rng;

    use crate::POWLUT;
    pub const DEFUALT_ITERATIONS: usize = 1000000;

    #[derive(Default)]
    pub struct Report {
        pub max_relative_error: f64,
        pub max_error_orig: DVec3,
        pub max_error_xyz_decoded: DVec3,

        pub max_abs_error: f64,
        pub max_abs_error_orig: DVec3,
        pub max_abs_error_decoded: DVec3,

        pub max_nor_dist: f64,
        pub max_nor_dist_orig: DVec3,
        pub max_nor_dist_decoded: DVec3,

        pub max_dist: f64,
        pub max_dist_orig: DVec3,
        pub max_dist_decoded: DVec3,

        pub avg_dist: f64,
    }

    impl Report {
        pub fn new<F>(distance: f64, iterations: usize, unsigned: bool, proc: F) -> Report
        where
            F: Fn(DVec3) -> DVec3,
        {
            let mut report = Report::default();
            let mut rng = rand::thread_rng();

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

                    let a = DVec3::from(orig);
                    let b = DVec3::from(decoded);
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
            report.avg_dist /= iterations as f64;
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

    pub fn report_set<F>(n: usize, proc: F) -> Vec<(f64, Report)>
    where
        F: Fn(DVec3) -> DVec3,
    {
        let mut set = Vec::new();
        for i in 1..57 {
            let mut n = i as f64 * 0.25;
            n = n.exp2() - 1.0;
            let report = Report::new(n, DEFUALT_ITERATIONS, false, |v| proc(v));
            set.push((n, report));
        }
        set
    }

    pub fn typ_ranges<F>(iterations: usize, max: f64, proc: F) -> Vec<(f64, Report)>
    where
        F: Fn(DVec3) -> DVec3,
    {
        let mut set = Vec::new();
        for i in 0..6 {
            let n = POWLUT[i];
            if n > max {
                break;
            }
            let report = Report::new(n, iterations, false, |v| proc(v));
            set.push((n, report));
        }
        set
    }
}
