#ifndef BSDF_BRDF_HLSLI
#define BSDF_BRDF_HLSLI

#include "../random/source.hlsli"

struct BrdfOutput {
    float3 next_dir;
    float3 reflectance;
    float pdf;
};

#endif // BSDF_BRDF_HLSLI
