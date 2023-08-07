const uint XYZ18E7_EXPONENT_BITS        = 7u;
const int  XYZ18E7_MANTISSA_BITS        = 18;
const uint XYZ18E7_MANTISSA_BITSU       = 18u;
const int  XYZ18E7_EXP_BIAS             = 63;
const uint XYZ18E7_MAX_VALID_BIASED_EXP = 127u;

const uint  MAX_XYZ18E7_EXP             = 64u;
const int   XYZ18E7_MANTISSA_VALUES     = 262144;
const int   MAX_XYZ18E7_MANTISSA        = 262143;
const uint  MAX_XYZ18E7_MANTISSAU       = 262143u;
const float MAX_XYZ18E7_                = 1.8446674e+19;
const float EPSILON_XYZ18E7_            = 4.135903e-25;

int floor_log2(float x) {
    uint f = floatBitsToUint(x);
    uint biasedexponent = (f & 0x7F800000u) >> 23u;
    return int(biasedexponent) - 127;
}

uint is_sign_negative(float v) {
    return (floatBitsToUint(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
uvec2 vec3_to_xyz18e7(vec3 xyz) {
    uint xsign = is_sign_negative(xyz.x);
    uint ysign = is_sign_negative(xyz.y);
    uint zsign = is_sign_negative(xyz.z);

    xyz = min(abs(xyz), vec3(MAX_XYZ18E7_));

    float maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    int exp_shared = max(-XYZ18E7_EXP_BIAS - 1, floor_log2(maxxyz)) + 1 + XYZ18E7_EXP_BIAS;
    float denom = exp2(float(exp_shared - XYZ18E7_EXP_BIAS - XYZ18E7_MANTISSA_BITS));

    int maxm = int(floor(maxxyz / denom + 0.5));
    if (maxm == XYZ18E7_MANTISSA_VALUES) {
        denom *= 2.0;
        exp_shared += 1;
    }

    uvec3 s = uvec3(floor(xyz / denom + 0.5));

    uint out_a = (s.y << 18u) | (s.x << 0u);
    uint out_b = (uint(exp_shared) << 25u) | (zsign << 24u) | (ysign << 23u) | (xsign << 22u) | (s.z << 4u) | (s.y >> 14u);
    
    return uvec2(out_a, out_b);
}

// Provided for compatibility. With opengl 4.0 and later bitfieldExtract can be used instead.
uint bitfield_extract(uint value, uint offset, uint bits) {
    uint mask = (1u << bits) - 1u;
    return (value >> offset) & mask;
}

vec3 xyz18e7_to_vec3(uvec2 v) {
    int exponent = int(bitfield_extract(v.y, 25u, XYZ18E7_EXPONENT_BITS)) - XYZ18E7_EXP_BIAS - XYZ18E7_MANTISSA_BITS;
    float scale = exp2(float(exponent));

    uint xb = bitfield_extract(v.x, 0u, XYZ18E7_MANTISSA_BITSU);
    uint yb = bitfield_extract(v.x, 18u, XYZ18E7_MANTISSA_BITSU) | bitfield_extract(v.y, 0u, 4u) << 14u;
    uint zb = bitfield_extract(v.y, 4u, XYZ18E7_MANTISSA_BITSU);

    // Extract the sign bits, then bitfield_extract(v.y, 22u, 1u) << 31u shifts it over to the corresponding IEEE 754 sign location.
    return vec3(
        uintBitsToFloat(floatBitsToUint(float(xb)) | bitfield_extract(v.y, 22u, 1u) << 31u),
        uintBitsToFloat(floatBitsToUint(float(yb)) | bitfield_extract(v.y, 23u, 1u) << 31u),
        uintBitsToFloat(floatBitsToUint(float(zb)) | bitfield_extract(v.y, 24u, 1u) << 31u)
    ) * scale;
}