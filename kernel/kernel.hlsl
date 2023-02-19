[[vk::binding(0)]] RWTexture2D<float4> output;

[numthreads(8, 8, 1)]
void cs_main(in uint2 pixel_coord : SV_DispatchThreadID) {
    output[pixel_coord] = float4(1.0, 0.0, 0.0, 1.0);
}
