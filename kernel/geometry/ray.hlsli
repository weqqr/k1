#ifndef GEOMETRY_RAY_HLSLI
#define GEOMETRY_RAY_HLSLI

class Ray {
    float3 origin;
    float3 dir;
    float3 inv_dir;

    float3 point_at(float t) {
        return origin + t*dir;
    }
};

Ray NewRay(float3 origin, float3 dir) {
    float3 inv_dir = 1.0 / dir;

    Ray ray;
    ray.origin = origin;
    ray.dir = dir;
    ray.inv_dir = inv_dir;

    return ray;
}

#endif // GEOMETRY_RAY_HLSLI
