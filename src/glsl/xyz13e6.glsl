const uint XYZ13E6_EXPONENT_BITS        = 6u;
const int  XYZ13E6_MANTISSA_BITS        = 13;
const uint XYZ13E6_MANTISSA_BITSU       = 13u;
const int  XYZ13E6_EXP_BIAS             = 31;
const uint XYZ13E6_MAX_VALID_BIASED_EXP = 63u;

const uint  MAX_XYZ13E6_EXP             = 32u;
const int   XYZ13E6_MANTISSA_VALUES     = 8192;
const int   MAX_XYZ13E6_MANTISSA        = 8191;
const uint  MAX_XYZ13E6_MANTISSAU       = 8191u;
const float MAX_XYZ13E6_                = 4294443000.0;
const float EPSILON_XYZ13E6_            = 0.00000000000005684342;

int floor_log2(float x) {
    uint f = floatBitsToUint(x);
    uint biasedexponent = (f & 0x7F800000u) >> 23u;
    return int(biasedexponent) - 127;
}

uint is_sign_negative(float v) {
    return (floatBitsToUint(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
uvec2 vec3_to_xyz13e6(vec3 xyz) {
    uint xsign = is_sign_negative(xyz.x);
    uint ysign = is_sign_negative(xyz.y);
    uint zsign = is_sign_negative(xyz.z);

    xyz = min(abs(xyz), vec3(MAX_XYZ13E6_));

    float maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    int exp_shared = max(-XYZ13E6_EXP_BIAS - 1, floor_log2(maxxyz)) + 1 + XYZ13E6_EXP_BIAS;
    float denom = exp2(float(exp_shared - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS));

    int maxm = int(floor(maxxyz / denom + 0.5));
    if (maxm == XYZ13E6_MANTISSA_VALUES) {
        denom *= 2.0;
        exp_shared += 1;
    }

    uvec3 s = uvec3(floor(xyz / denom + 0.5));

    uint out_a = (uint(exp_shared) << 26u) | (s.y << 13u) | (s.x << 0u);
    uint out_b = (zsign << 15u) | (ysign << 14u) | (xsign << 13u) | (s.z << 0u);
    
    return uvec2(out_a, out_b);
}

// Provided for compatibility. With opengl 4.0 and later bitfieldExtract can be used instead.
uint bitfield_extract(uint value, uint offset, uint bits) {
    uint mask = (1u << bits) - 1u;
    return (value >> offset) & mask;
}

vec3 xyz13e6_to_vec3(uvec2 v) {
    int exponent = int(bitfield_extract(v.x, 26u, XYZ13E6_EXPONENT_BITS)) - XYZ13E6_EXP_BIAS - XYZ13E6_MANTISSA_BITS;
    float scale = exp2(float(exponent));

    // bitfield_extract(v.y, 13u, 1u) << 31u is extracting the sign bit and 
    // shifts it over to the corresponding IEEE 754 sign location 
    return vec3(
        uintBitsToFloat(floatBitsToUint(float(bitfield_extract(v.x,  0u, XYZ13E6_MANTISSA_BITSU))) | bitfield_extract(v.y, 13u, 1u) << 31u),
        uintBitsToFloat(floatBitsToUint(float(bitfield_extract(v.x, 13u, XYZ13E6_MANTISSA_BITSU))) | bitfield_extract(v.y, 14u, 1u) << 31u),
        uintBitsToFloat(floatBitsToUint(float(bitfield_extract(v.y,  0u, XYZ13E6_MANTISSA_BITSU))) | bitfield_extract(v.y, 15u, 1u) << 31u)
    ) * scale;
}