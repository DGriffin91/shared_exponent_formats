use crate::nan_to_zero;

#[derive(Debug, Clone, Copy)]
pub struct SharedExponentFormat {
    pub exponent_bits: u8,
    pub mantissa_bits: u8,
    pub max_valid_biased_exp: i32,
    pub exp_bias: i32,
    pub mantissa_values: i32,
    pub max_mantissa: i32,
    pub max_exp: i32,
    pub epsilon: f32,
    pub max: f32,
}

impl SharedExponentFormat {
    pub fn new(exponent_bits: u8, mantissa_bits: u8) -> Self {
        // TODO 8 exponent_bits should be possible
        debug_assert!(exponent_bits <= 7);
        debug_assert!(mantissa_bits <= 24);

        debug_assert!(exponent_bits > 0);
        debug_assert!(mantissa_bits > 0);

        // TODO this shouldn't probably be a restriction
        debug_assert!(exponent_bits * mantissa_bits <= 133);

        let max_valid_biased_exp = (1 << exponent_bits) - 1;
        let exp_bias = max_valid_biased_exp / 2;
        let mantissa_values = 1 << mantissa_bits;
        let max_mantissa = mantissa_values - 1;

        let max_exp = max_valid_biased_exp - exp_bias;

        let epsilon = (1.0 / mantissa_values as f32) / (1u128 << exp_bias as u128) as f32;
        //let epsilon = 1.0 / mantissa_values as f32 / 2.0f32.powi(exp_bias);

        let max = ((max_mantissa as f32) / mantissa_values as f32
            * (1u128 << max_exp as u128) as f32)
            .floor();

        //let mut max: f32 = (max_mantissa as f32) / mantissa_values as f32 * 2.0f32.powi(max_exp);
        //max = max.min(f32::MAX).floor();

        SharedExponentFormat {
            exponent_bits,
            mantissa_bits,
            max_valid_biased_exp,
            exp_bias,
            mantissa_values,
            max_mantissa,
            max_exp,
            epsilon,
            max,
        }
    }

    // Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
    #[inline]
    pub fn get_exp(&self, maxrgb: f32) -> (f32, u8) {
        let maxrgb = nan_to_zero(maxrgb).clamp(0.0, self.max);

        let mut exp_shared =
            (-self.exp_bias - 1).max(maxrgb.log2().floor() as i32) + 1 + self.exp_bias;

        debug_assert!(exp_shared <= self.max_valid_biased_exp);
        debug_assert!(exp_shared >= 0);

        let mut denom = ((exp_shared - self.exp_bias - self.mantissa_bits as i32) as f32).exp2();

        let maxm = (maxrgb / denom + 0.5).floor() as i32;
        if maxm == self.mantissa_values {
            denom *= 2.0;
            exp_shared += 1;
            exp_shared = exp_shared.min(self.max_valid_biased_exp);
            debug_assert!(exp_shared <= self.max_valid_biased_exp);
        } else {
            debug_assert!(maxm <= self.max_mantissa);
        }

        (denom, exp_shared as u8)
    }

    #[inline]
    pub fn norm(&self, denom: f32, v: f32) -> u32 {
        (nan_to_zero(v).clamp(0.0, self.max) / denom + 0.5).floor() as u32
    }

    #[inline]
    pub fn apply_exp(&self, v: u32, exp_shared: u8) -> f32 {
        let max_valid_biased_exp = (1 << self.exponent_bits) - 1;
        let exp_bias = max_valid_biased_exp / 2;

        let exponent = exp_shared as i32 - exp_bias - self.mantissa_bits as i32;
        let scale = (exponent as f32).exp2();

        v as f32 * scale
    }

    pub fn encode3(&self, v: [f32; 3]) -> ([u32; 3], u8) {
        let x = nan_to_zero(v[0]).clamp(0.0, self.max);
        let y = nan_to_zero(v[1]).clamp(0.0, self.max);
        let z = nan_to_zero(v[2]).clamp(0.0, self.max);
        let (denom, exp_shared) = self.get_exp(x.max(y).max(z));
        (
            [
                self.norm(denom, x),
                self.norm(denom, y),
                self.norm(denom, z),
            ],
            exp_shared,
        )
    }

    pub fn decode3(&self, enc: [u32; 3], exp_shared: u8) -> [f32; 3] {
        [
            self.apply_exp(enc[0], exp_shared),
            self.apply_exp(enc[1], exp_shared),
            self.apply_exp(enc[2], exp_shared),
        ]
    }
}

#[cfg(test)]
pub mod tests {

    use glam::Vec3;

    use super::*;

    #[test]
    fn test_edge_cases() {
        for exponent_bits in 1..=7 {
            for mantissa_bits in 1..=19 {
                if exponent_bits * mantissa_bits > 133 {
                    continue;
                }
                let format = SharedExponentFormat::new(exponent_bits, mantissa_bits);

                let (enc, exp_shared) = format.encode3(Vec3::ONE.into());
                debug_assert_eq!(Vec3::ONE, format.decode3(enc, exp_shared).into());

                let (enc, exp_shared) = format.encode3(Vec3::INFINITY.into());
                debug_assert_eq!(
                    Vec3::splat(format.max),
                    format.decode3(enc, exp_shared).into()
                );

                let (enc, exp_shared) = format.encode3((-Vec3::INFINITY).into());
                debug_assert_eq!(Vec3::ZERO, format.decode3(enc, exp_shared).into());

                let (enc, exp_shared) = format.encode3(Vec3::MAX.into());
                debug_assert_eq!(
                    Vec3::splat(format.max),
                    format.decode3(enc, exp_shared).into()
                );

                let (enc, exp_shared) = format.encode3((-Vec3::MAX).into());
                debug_assert_eq!(Vec3::ZERO, format.decode3(enc, exp_shared).into());

                let (enc, exp_shared) = format.encode3((-Vec3::ZERO).into());
                debug_assert_eq!(Vec3::ZERO, format.decode3(enc, exp_shared).into());

                let (enc, exp_shared) = format.encode3((-Vec3::NAN).into());
                debug_assert_eq!(Vec3::ZERO, format.decode3(enc, exp_shared).into());
            }
        }
    }
}
