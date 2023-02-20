#ifndef GEOMETRY_SPHERE_HLSLI
#define GEOMETRY_SPHERE_HLSLI

#include "intersection.hlsli"
#include "ray.hlsli"

class Sphere {
    float3 center;
    float radius;

    IntersectionData intersect(in Ray ray) {
        IntersectionData data;

        float3 oc = ray.origin - center;
        float b = dot(oc, ray.dir);
        float c = dot(oc, oc) - radius*radius;
        float h = b*b - c;
        if (h < 0.0) {
            data.t = -1.0;
            return data;
        }
        h = sqrt(h);
        float2 nf = float2(-b-h, -b+h);
        if (nf.x < nf.y) {
            data.t = nf.x;
            data.pt = ray.point_at(data.t);
            data.normal = normalize(data.pt - center);
        }

        return data;
    }

    float3 normal(in float3 pt) {
        return normalize(pt - center);
    }
};

#endif // GEOMETRY_SPHERE_HLSLI
