#ifndef INTEGRATOR_PATH_TRACER_HLSLI
#define INTEGRATOR_PATH_TRACER_HLSLI

#include "../bsdf/brdf.hlsli"
#include "../bsdf/lambertian.hlsli"
#include "../geometry/ray.hlsli"
#include "../geometry/sphere.hlsli"
#include "../random/source.hlsli"

#define HIGHLIGHT_BIASED_PATHS 1

#define MAX_BOUNCES 4

class PathTracer {
    float3 integrate(
        in Ray ray,
        inout RandomSource random,
        in Lambertian brdf,
        in Sphere sphere
    ) {
        float3 color = float3(0.0, 0.0, 0.0);
        float3 throughput = float3(1.0, 1.0, 1.0);

        float3 sky = float3(1.0, 1.0, 1.0);
        bool last_diffuse = false;
        for (int i = 0; i < MAX_BOUNCES; i++) {
            IntersectionData data = sphere.intersect(ray);

            if (dot(data.normal, ray.dir) > 0) {
                data.normal = -data.normal;
            }

            if (data.t < 0) {
                color += throughput * sky;
                return color;
            }

            BrdfOutput result = brdf.evaluate(random, data.normal, ray.dir);

            throughput *= result.reflectance / result.pdf;

            ray = NewRay(data.pt, result.next_dir);
        }

#if HIGHLIGHT_BIASED_PATHS
        return float3(1, 0, 1);
#else
        return color;
#endif
    }
};

#endif // INTEGRATOR_PATH_TRACER_HLSLI
