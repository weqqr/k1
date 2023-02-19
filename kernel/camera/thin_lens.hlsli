#ifndef CAMERA_THIN_LENS_HLSLI
#define CAMERA_THIN_LENS_HLSLI

#include "../geometry/ray.hlsli"
#include "../geometry/vector.hlsli"

class ThinLensCamera {
    float3 origin;
    float3 lc;
    float3 up;
    float3 right;

    Ray get_ray(float2 uv) {
        float3 r = right * uv.x;
        float3 u = up * uv.y;
        return NewRay(origin, normalize((lc + r + u - origin)));
    }
};

ThinLensCamera NewCameraFromVectors(in float3 origin, in float3 forward, in float fov, in float aspect_ratio) {
    ThinLensCamera camera;

    float3 right = normalize(cross(forward, Y));
    float3 up = normalize(cross(right, forward));

    float half_width = tan(fov / 2);
    float half_height = half_width / aspect_ratio;

    float3 half_up_scaled = up * half_height;
    float3 half_right_scaled = right * half_width;
    float3 offset = half_up_scaled + half_right_scaled;

    camera.origin = origin;
    camera.lc = camera.origin + forward - offset;
    camera.up = up * 2 * half_height;
    camera.right = right * 2 * half_width;

    return camera;
}

ThinLensCamera NewCameraFromTransform(in float4x4 transform, in float fov, in float aspect_ratio) {
    float3 origin = mul(float4(0, 0, 0, 1), transform).xyz;
    float3x4 ray_tf = float3x4(
        transform[0],
        transform[1],
        transform[2]
    );

    float3 forward = normalize(mul(float3(0, 0, 1), ray_tf)).xyz;

    return NewCameraFromVectors(origin, forward, fov, aspect_ratio);
}

#endif // CAMERA_THIN_LENS_HLSLI
