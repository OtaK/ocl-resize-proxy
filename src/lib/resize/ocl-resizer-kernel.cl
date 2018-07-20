__constant sampler_t samplerIn =
    CLK_NORMALIZED_COORDS_FALSE |
    CLK_ADDRESS_CLAMP |
    CLK_FILTER_LINEAR;

__kernel void resizeImage(
    read_only  image2d_t sourceImage,
    write_only image2d_t targetImage
) {
    float2 posIn = convert_float2((int2)(get_global_id(0), get_global_id(1)));
    int2 target_dim = get_image_dim(targetImage);
    int2 source_dim = get_image_dim(sourceImage);
    int2 posOut = {
        ((double)posIn.x + 0.49995f) * ((double) target_dim.x / (double) source_dim.x),
        ((double)posIn.y + 0.49995f) * ((double) target_dim.y / (double) source_dim.y)
    };
    float4 pixel = read_imagef(sourceImage, samplerIn, posIn);
    write_imagef(targetImage, posOut, pixel);
}
