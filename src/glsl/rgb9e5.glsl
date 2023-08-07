const uint RGB9E5_EXPONENT_BITS        = 5u;
const int  RGB9E5_MANTISSA_BITS        = 9;
const uint RGB9E5_MANTISSA_BITSU       = 9u;
const int  RGB9E5_EXP_BIAS             = 15;
const uint RGB9E5_MAX_VALID_BIASED_EXP = 31u;

const uint  MAX_RGB9E5_EXP             = 16u;
const int   RGB9E5_MANTISSA_VALUES     = 512;
const int   MAX_RGB9E5_MANTISSA        = 511;
const uint  MAX_RGB9E5_MANTISSAU       = 511u;
const float MAX_RGB9E5_                = 65408.0;
const float EPSILON_RGB9E5_            = 0.000000059604645;

int floor_log2(float x) {
    uint f = floatBitsToUint(x);
    uint biasedexponent = (f & 0x7F800000u) >> 23u;
    return int(biasedexponent) - 127;
}

// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
uint vec3_to_rgb9e5(vec3 rgb_in) {
    vec3 rgb = clamp(rgb_in, vec3(0.0), vec3(MAX_RGB9E5_));

    float maxrgb = max(rgb.r, max(rgb.g, rgb.b));
    int exp_shared = max(-RGB9E5_EXP_BIAS - 1, floor_log2(maxrgb)) + 1 + RGB9E5_EXP_BIAS;
    float denom = exp2(float(exp_shared - RGB9E5_EXP_BIAS - RGB9E5_MANTISSA_BITS));

    int maxm = int(floor(maxrgb / denom + 0.5));
    if (maxm == RGB9E5_MANTISSA_VALUES) {
        denom *= 2.0;
        exp_shared += 1;
    }

    uvec3 n = uvec3(floor(rgb / denom + 0.5));
    
    return (uint(exp_shared) << 27u) | (n.b << 18u) | (n.g << 9u) | (n.r << 0u);
}

// Provided for compatibility. With opengl 4.0 and later bitfieldExtract can be used instead.
uint bitfield_extract(uint value, uint offset, uint bits) {
    uint mask = (1u << bits) - 1u;
    return (value >> offset) & mask;
}

vec3 rgb9e5_to_vec3(uint v) {
    int exponent = int(bitfield_extract(v, 27u, RGB9E5_EXPONENT_BITS)) - RGB9E5_EXP_BIAS - RGB9E5_MANTISSA_BITS;
    float scale = exp2(float(exponent));

    return vec3(
        float((v >> 0u) & ((1u << RGB9E5_MANTISSA_BITSU) - 1u)),
        float((v >> 9u) & ((1u << RGB9E5_MANTISSA_BITSU) - 1u)),
        float((v >> 18u) & ((1u << RGB9E5_MANTISSA_BITSU) - 1u))
    ) * scale;
}