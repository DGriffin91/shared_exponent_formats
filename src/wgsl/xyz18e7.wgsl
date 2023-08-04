const XYZ13E6_EXPONENT_BITS        = 7u;
const XYZ13E6_MANTISSA_BITS        = 18;
const XYZ13E6_MANTISSA_BITSU       = 18u;
const XYZ13E6_EXP_BIAS             = 63;
const XYZ13E6_MAX_VALID_BIASED_EXP = 127u;

const MAX_XYZ13E6_EXP              = 64u;
const XYZ13E6_MANTISSA_VALUES      = 262144;
const MAX_XYZ13E6_MANTISSA         = 262143;
const MAX_XYZ13E6_MANTISSAU        = 262143u;
const MAX_XYZ13E6_                 = 1.8446674e+19;
const EPSILON_XYZ13E6_             = 4.135903e-25;

fn floor_log2_(x: f32) -> i32 {
    let f = bitcast<u32>(x);
    let biasedexponent = (f & 0x7F800000u) >> 23u;
    return i32(biasedexponent) - 127;
}

fn is_sign_negative(v: f32) -> u32 {
    return (bitcast<u32>(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
fn vec3_to_xyz18e7_(xyz: vec3<f32>) -> vec2<u32> {
    let xsign = is_sign_negative(xyz.x);
    let ysign = is_sign_negative(xyz.y);
    let zsign = is_sign_negative(xyz.z);

    var xyz = min(abs(xyz), vec3(MAX_XYZ13E6_));

    let maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    var exp_shared = max(-XYZ13E6_EXP_BIAS - 1, floor_log2_(maxxyz)) + 1 + XYZ13E6_EXP_BIAS;
    var denom = exp2(f32(exp_shared - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS));

    let maxm = i32(floor(maxxyz / denom + 0.5));
    if (maxm == XYZ13E6_MANTISSA_VALUES) {
        denom *= 2.0;
        exp_shared += 1;
    }

    let s = vec3<u32>(floor(xyz / denom + 0.5));

    let out_a = (s.y << 18u) | (s.x << 0u);
    let out_b = (u32(exp_shared) << 25u) | (zsign << 24u) | (ysign << 23u) | (xsign << 22u) | (s.z << 4u) | (s.y >> 14u);
    
    return vec2(out_a, out_b);
}

fn xyz18e7_to_vec3_(v: vec2<u32>) -> vec3<f32> {
    let exponent = i32(extractBits(v[1], 25u, XYZ13E6_EXPONENT_BITS)) - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS;
    let scale = exp2(f32(exponent));

    
    let xb = extractBits(v[0], 0u, XYZ13E6_MANTISSA_BITSU);
    let yb = extractBits(v[0], 18u, XYZ13E6_MANTISSA_BITSU) | extractBits(v[1], 0u, 4u) << 14u;
    let zb = extractBits(v[1], 4u, XYZ13E6_MANTISSA_BITSU);

    // Extract the sign bits, then extractBits(v[1], 22u, 1u) << 31u shifts it over to the corresponding IEEE 754 sign location.
    return vec3(
        bitcast<f32>(bitcast<u32>(f32(xb)) | extractBits(v[1], 22u, 1u) << 31u),
        bitcast<f32>(bitcast<u32>(f32(yb)) | extractBits(v[1], 23u, 1u) << 31u),
        bitcast<f32>(bitcast<u32>(f32(zb)) | extractBits(v[1], 24u, 1u) << 31u),
    ) * scale;
}