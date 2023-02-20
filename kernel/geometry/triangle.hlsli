#ifndef GEOMETRY_TRIANGLE_HLSLI
#define GEOMETRY_TRIANGLE_HLSLI

#include "../math/constants.hlsli"
#include "intersection.hlsli"
#include "ray.hlsli"

class Triangle {
    float3 v0, v1, v2;
    float3 n0, n1, n2;

    IntersectionData intersect(in Ray ray) {
        float3 b = v2 - v0;
        float3 a = v0 - v1;
        float3 p = v0 - ray.origin;
        float3 n = cross(b, a);
        float3 q = cross(p, ray.dir);

        float idet = 1.0/dot(ray.dir, n);

        float u = dot(q, b)*idet;
        float v = dot(q, a)*idet;
        float t = dot(n, p)*idet;

        if (u < 0.0 || v < 0.0 || (u+v) > 1.0) {
            t = -1.0;
        }

        float w = 1 - u - v;

        IntersectionData data;

        // Avoid self-intersections on next ray bounce
        data.t = t - EPSILON;
        data.pt = ray.point_at(data.t);
        data.normal = normalize(cross(v2-v0, v0-v1));

        return data;
    }

    float3 normal() {
        return normalize(cross(v2-v0, v0-v1));
    }
};

#endif // GEOMETRY_TRIANGLE_HLSLI
