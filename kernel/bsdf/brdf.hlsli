#ifndef BSDF_BRDF_HLSLI
#define BSDF_BRDF_HLSLI

struct BrdfOutput {
    float3 next_dir;
    float3 reflectance;
    float pdf;
};

#endif // BSDF_BRDF_HLSLI
