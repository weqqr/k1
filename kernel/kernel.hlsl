#include "geometry/ray.hlsli"
#include "geometry/sphere.hlsli"

[[vk::binding(0)]] RWTexture2D<float4> output;

static const float2 OUTPUT_SIZE = float2(1280.0, 720.0);

[numthreads(8, 8, 1)]
void cs_main(in uint2 pixel_coord : SV_DispatchThreadID) {
    float2 p = float2(pixel_coord) / OUTPUT_SIZE;

    Sphere sphere;
    sphere.center = float3(0.0, 0.0, -1.0);
    sphere.radius = 0.5;

    float3 origin = float3(0.0, 0.0, 0.0);
    float3 dir = normalize(float3(p.x, p.y, -1.0));

    Ray ray = NewRay(origin, dir);

    float4 color;
    if (sphere.intersect(ray) > 0) {
        color = float4(p.x, p.y, 0.0, 1.0);
    } else {
        color = float4(0.0, 0.0, 0.0, 1.0);
    }

    output[pixel_coord] = color;
}
