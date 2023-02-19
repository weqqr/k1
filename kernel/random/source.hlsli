#ifndef RANDOM_SOURCE_HLSLI
#define RANDOM_SOURCE_HLSLI

#include "../math/constants.hlsli"

uint xoroshiro64_rotl(uint x, uint k) {
    return (x << k) | (x >> (32 - k));
}

float bits_to_normalized_float(in uint bits) {
    return asfloat((bits & 0x007FFFFFu) | 0x3F800000u) - 1.0;
}

class RandomSource {
    uint2 state;
    float s;

    uint xoroshiro64() {
        uint result = state[0] * 0x9E3779BB;

        state[1] ^= state[0];
        state[0] = xoroshiro64_rotl(state[0], 26) ^ state[1] ^ (state[1] << 9); // a, b
        state[1] = xoroshiro64_rotl(state[1], 13); // c

        return result;
    }

    float random() {
        return bits_to_normalized_float(xoroshiro64());
    }

    float3 uniform_on_sphere() {
        float phi = 2 * PI * random();
        float cos_theta = 2 * random() - 1;
        float sin_theta = sqrt(1 - cos_theta * cos_theta);

        return float3(sin_theta*cos(phi), cos_theta, sin_theta*sin(phi));
    }

    float3 cosine_weighted_on_hemisphere(in float3 normal) {
        return normalize(normal + uniform_on_sphere());
    }
};

RandomSource NewRandomSource(uint2 seed) {
    RandomSource source;
    source.state = seed;
    return source;
}

#endif // RANDOM_SOURCE_HLSLI
