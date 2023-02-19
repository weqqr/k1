#include "bsdf/lambertian.hlsli"
#include "camera/thin_lens.hlsli"
#include "geometry/ray.hlsli"
#include "geometry/sphere.hlsli"
#include "integrator/path_tracer.hlsli"

[[vk::binding(0)]] RWTexture2D<float4> output;

static const float2 OUTPUT_SIZE = float2(1280.0, 720.0);

[numthreads(8, 8, 1)]
void cs_main(in uint2 pixel_coord : SV_DispatchThreadID) {
    float2 p = float2(pixel_coord) / OUTPUT_SIZE;
    // wgpu: Invert Y so it's positive on top
    p.y = 1 - p.y;

    float3 origin = float3(0.0, 0.0, 0.0);
    float3 dir = normalize(float3(0.0, 0.0, -1.0));
    float aspect_ratio = OUTPUT_SIZE.x / OUTPUT_SIZE.y;
    ThinLensCamera camera = NewCameraFromVectors(origin, dir, radians(60.0), aspect_ratio);

    RandomSource random = NewRandomSource(pixel_coord);

    Sphere sphere;
    sphere.center = float3(0.0, 0.0, -1.0);
    sphere.radius = 0.5;

    Lambertian brdf;
    brdf.albedo = float3(1.0, 1.0, 1.0);

    Ray ray = camera.get_ray(p);

    PathTracer integrator;

    float3 color = integrator.integrate(ray, random, brdf, sphere);

    output[pixel_coord] = float4(color, 1.0);
}
