#include "camera/thin_lens.hlsli"
#include "geometry/ray.hlsli"
#include "geometry/sphere.hlsli"

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

    Sphere sphere;
    sphere.center = float3(0.0, 0.0, -1.0);
    sphere.radius = 0.5;

    Ray ray = camera.get_ray(p);

    float4 color;
    float t = sphere.intersect(ray);
    if (t > 0) {
        float3 normal = sphere.normal(ray.point_at(t));
        color = float4(normal, 1.0);
    } else {
        color = float4(0.0, 0.0, 0.0, 1.0);
    }

    output[pixel_coord] = color;
}
