const XYZ13E6_EXPONENT_BITS        = 6u;
const XYZ13E6_MANTISSA_BITS        = 13;
const XYZ13E6_MANTISSA_BITSU       = 13u;
const XYZ13E6_EXP_BIAS             = 31;
const XYZ13E6_MAX_VALID_BIASED_EXP = 63u;

const MAX_XYZ13E6_EXP              = 32u;
const XYZ13E6_MANTISSA_VALUES      = 8192u;
const MAX_XYZ13E6_MANTISSA         = 8191;
const MAX_XYZ13E6_MANTISSAU        = 8191u;
const MAX_XYZ13E6_                 = 4294443000.0;
const EPSILON_XYZ13E6_             = 0.00000000000005684342;

fn floor_log2_(x: f32) -> i32 {
    let f = bitcast<u32>(x);
    let biasedexponent = (f & 0x7F800000u) >> 23u;
    return i32(biasedexponent) - 127;
}

fn is_sign_positive(v: f32) -> u32 {
    return ~(bitcast<u32>(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
fn vec3_to_xyz13e6_(xyz: vec3<f32>) -> vec2<u32> {
    let xsign = is_sign_positive(xyz.x);
    let ysign = is_sign_positive(xyz.y);
    let zsign = is_sign_positive(xyz.z);

    var xyz = min(abs(xyz), vec3(MAX_XYZ13E6_));

    let maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    var exp_shared = max(-XYZ13E6_EXP_BIAS - 1, floor_log2_(maxxyz)) + 1 + XYZ13E6_EXP_BIAS;
    var denom = exp2(f32(exp_shared - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS));

    let maxm = i32(floor(maxxyz / denom + 0.5));
    if (maxm == MAX_XYZ13E6_MANTISSA + 1) {
        denom *= 2.0;
        exp_shared += 1;
    }

    let s = floor(xyz / denom + 0.5);

    let xm = u32(s.x);
    let ym = u32(s.y);
    let zm = u32(s.z);

    let out_a = (u32(exp_shared) << 26u) | (ym << 13u) | (xm << 0u);
    let out_b = (zsign << 15u) | (ysign << 14u) | (xsign << 13u) | (zm << 0u);
    
    return vec2(out_a, out_b);
}

fn xyz13e6_to_vec3_(v: vec2<u32>) -> vec3<f32> {
    let exponent = i32(extractBits(v[0], 26u, XYZ13E6_EXPONENT_BITS)) - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS;
    let scale = exp2(f32(exponent));

    let xsign = f32(extractBits(v[1], 13u, 1u) << 1u) - 1.0;
    let ysign = f32(extractBits(v[1], 14u, 1u) << 1u) - 1.0;
    let zsign = f32(extractBits(v[1], 15u, 1u) << 1u) - 1.0;

    return vec3(
        xsign * f32(extractBits(v[0], 0u, XYZ13E6_MANTISSA_BITSU)) * scale,
        ysign * f32(extractBits(v[0], 13u, XYZ13E6_MANTISSA_BITSU)) * scale,
        zsign * f32(extractBits(v[1], 0u, XYZ13E6_MANTISSA_BITSU)) * scale
    );
}