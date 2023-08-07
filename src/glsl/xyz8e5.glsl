const uint XYZ8E5_EXPONENT_BITS        = 5u;
const int  XYZ8E5_MANTISSA_BITS        = 8;
const uint XYZ8E5_MANTISSA_BITSU       = 8u;
const int  XYZ8E5_EXP_BIAS             = 15;
const uint XYZ8E5_MAX_VALID_BIASED_EXP = 31u;

const uint  MAX_XYZ8E5_EXP             = 16u;
const int   XYZ8E5_MANTISSA_VALUES     = 256;
const int   MAX_XYZ8E5_MANTISSA        = 255;
const uint  MAX_XYZ8E5_MANTISSAU       = 255u;
const float MAX_XYZ8E5_                = 65280.0;
const float EPSILON_XYZ8E5_            = 0.00000011920929;

int floor_log2(float x) {
    uint f = floatBitsToUint(x);
    uint biasedexponent = (f & 0x7F800000u) >> 23u;
    return int(biasedexponent) - 127;
}

uint is_sign_negative(float v) {
    return (floatBitsToUint(v) >> 31u) & 1u;
}

// Similar to https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
uint vec3_to_xyz8e5(vec3 xyz_in) {
    uint xsign = is_sign_negative(xyz_in.x) << 8u;
    uint ysign = is_sign_negative(xyz_in.y) << 8u;
    uint zsign = is_sign_negative(xyz_in.z) << 8u;

    vec3 xyz = min(abs(xyz_in), vec3(MAX_XYZ8E5_));

    float maxxyz = max(xyz.x, max(xyz.y, xyz.z));
    int exp_shared = max(-XYZ8E5_EXP_BIAS - 1, floor_log2(maxxyz)) + 1 + XYZ8E5_EXP_BIAS;
    float denom = exp2(float(exp_shared - XYZ8E5_EXP_BIAS - XYZ8E5_MANTISSA_BITS));

    int maxm = int(floor(maxxyz / denom + 0.5));
    if (maxm == XYZ8E5_MANTISSA_VALUES) {
        denom *= 2.0;
        exp_shared += 1;
    }

    uvec3 s = uvec3(floor(xyz / denom + 0.5));

    return (uint(exp_shared) << 27u) | ((s.z | zsign) << 18u) | ((s.y | ysign) << 9u) | ((s.x | xsign) << 0u);
}

// Provided for compatibility. With opengl 4.0 and later bitfieldExtract can be used instead.
uint bitfield_extract(uint value, uint offset, uint bits) {
    uint mask = (1u << bits) - 1u;
    return (value >> offset) & mask;
}

vec3 xyz8e5_to_vec3(uint v) {
    int exponent = int(bitfield_extract(v, 27u, XYZ8E5_EXPONENT_BITS)) - XYZ8E5_EXP_BIAS - XYZ8E5_MANTISSA_BITS;
    float scale = exp2(float(exponent));

    // Extract both the mantissa and sign at the same time.
    uint xb = bitfield_extract(v,  0u, XYZ8E5_MANTISSA_BITSU + 1u);
    uint yb = bitfield_extract(v,  9u, XYZ8E5_MANTISSA_BITSU + 1u);
    uint zb = bitfield_extract(v, 18u, XYZ8E5_MANTISSA_BITSU + 1u);

    // xb & 0xFFu masks out for just the mantissa
    // xb & 0x100u << 23u masks out just the sign bit and shifts it over 
    // to the corresponding IEEE 754 sign location 
    return vec3(
        uintBitsToFloat(floatBitsToUint(float(xb & 0xFFu)) | (xb & 0x100u) << 23u),
        uintBitsToFloat(floatBitsToUint(float(yb & 0xFFu)) | (yb & 0x100u) << 23u),
        uintBitsToFloat(floatBitsToUint(float(zb & 0xFFu)) | (zb & 0x100u) << 23u)
    ) * scale;
}