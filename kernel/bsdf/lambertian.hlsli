#ifndef BSDF_LAMBERTIAN_HLSLI
#define BSDF_LAMBERTIAN_HLSLI

#include "../math/constants.hlsli"
#include "../random/source.hlsli"
#include "brdf.hlsli"

class Lambertian {
    float3 albedo;

    BrdfOutput evaluate(inout RandomSource random, float3 normal, float3 wo) {
        BrdfOutput output;
        output.next_dir = random.cosine_weighted_on_hemisphere(normal);
        output.reflectance = albedo / PI;
        output.pdf = 1 / PI;

        return output;
    }
};

#endif // BSDF_LAMBERTIAN_HLSLI
