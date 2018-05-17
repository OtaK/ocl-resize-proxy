const sampler_t samplerIn =
    CLK_NORMALIZED_COORDS_TRUE |
    CLK_ADDRESS_CLAMP |
    CLK_FILTER_LINEAR;

const sampler_t samplerOut =
    CLK_NORMALIZED_COORDS_FALSE |
    CLK_ADDRESS_CLAMP |
    CLK_FILTER_NEAREST;

__kernel void resizeImage(
    __read_only  image2d_t sourceImage,
    __write_only image2d_t targetImage
) {
    int w = get_image_width(targetImage);
    int h = get_image_height(targetImage);

    int outX = get_global_id(0);
    int outY = get_global_id(1);
    int2 posOut = {outX, outY};

    float inX = outX / (float) w;
    float inY = outY / (float) h;
    float2 posIn = (float2) (inX, inY);

    float4 pixel = read_imagef(sourceImage, samplerIn, posIn);
    write_imagef(targetImage, posOut, pixel);
}
