const XYZ8E5_EXPONENT_BITS        = 5u;
const XYZ8E5_MANTISSA_BITS        = 8;
const XYZ8E5_MANTISSA_BITSU       = 8u;
const XYZ8E5_EXP_BIAS             = 15;
const XYZ8E5_MAX_VALID_BIASED_EXP = 31u;

const MAX_XYZ8E5_EXP              = 16u;
const XYZ8E5_MANTISSA_VALUES      = 256u;
const MAX_XYZ8E5_MANTISSA         = 255;
const MAX_XYZ8E5_MANTISSAU        = 255u;
const MAX_XYZ8E5_                 = 65280.0;
const EPSILON_XYZ8E5_             = 0.00000011920929;

fn floor_log2_(x: f32) -> i32 {
    let f = bitcast<u32>(x);
    let biasedexponent = (f & 0x7F800000u) >> 23u;
    return i32(biasedexponent) - 127;
}

fn is_sign_positive(v: f32) -> u32 {
    return ~(bitcast<u32>(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
fn vec3_to_xyz8e5_(xyz: vec3<f32>) -> u32 {
    let xsign = is_sign_positive(xyz.x);
    let ysign = is_sign_positive(xyz.y);
    let zsign = is_sign_positive(xyz.z);

    var xyz = min(abs(xyz), vec3(MAX_XYZ8E5_));

    let maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    var exp_shared = max(-XYZ8E5_EXP_BIAS - 1, floor_log2_(maxxyz)) + 1 + XYZ8E5_EXP_BIAS;
    var denom = exp2(f32(exp_shared - XYZ8E5_EXP_BIAS - XYZ8E5_MANTISSA_BITS));

    let maxm = i32(floor(maxxyz / denom + 0.5));
    if (maxm == MAX_XYZ8E5_MANTISSA + 1) {
        denom *= 2.0;
        exp_shared += 1;
    }

    let s = floor(xyz / denom + 0.5);

    let xm = u32(s.x) | xsign << 8u;
    let ym = u32(s.y) | ysign << 8u;
    let zm = u32(s.z) | zsign << 8u;
    
    return (u32(exp_shared) << 27u) | (zm << 18u) | (ym << 9u) | (xm << 0u);
}

fn xyz8e5_to_vec3_(v: u32) -> vec3<f32> {
    let exponent = i32(extractBits(v, 27u, XYZ8E5_EXPONENT_BITS)) - XYZ8E5_EXP_BIAS - XYZ8E5_MANTISSA_BITS;
    let scale = exp2(f32(exponent));

    let xsign = f32(extractBits(v, 8u, 1u) << 1u) - 1.0;
    let ysign = f32(extractBits(v, 17u, 1u) << 1u) - 1.0;
    let zsign = f32(extractBits(v, 26u, 1u) << 1u) - 1.0;

    return vec3(
        xsign * f32(extractBits(v, 0u, XYZ8E5_MANTISSA_BITSU)) * scale,
        ysign * f32(extractBits(v, 9u, XYZ8E5_MANTISSA_BITSU)) * scale,
        zsign * f32(extractBits(v, 18u, XYZ8E5_MANTISSA_BITSU)) * scale
    );
}