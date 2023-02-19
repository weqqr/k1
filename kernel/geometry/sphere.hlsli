#ifndef GEOMETRY_SPHERE_HLSLI
#define GEOMETRY_SPHERE_HLSLI

#include "ray.hlsli"

class Sphere {
    float3 center;
    float radius;

    float intersect(in Ray ray) {
        float3 oc = ray.origin - center;
        float b = dot(oc, ray.dir);
        float c = dot(oc, oc) - radius*radius;
        float h = b*b - c;
        if (h < 0.0) {
            return -1.0;
        }
        h = sqrt(h);
        float2 nf = float2(-b-h, -b+h);
        if (nf.x < nf.y) {
            return nf.x;
        }
        return -1.0;
    }

    float3 normal(in float3 pt) {
        return normalize(pt - center);
    }
};

#endif // GEOMETRY_SPHERE_HLSLI
