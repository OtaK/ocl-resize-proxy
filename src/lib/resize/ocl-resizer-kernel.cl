__constant sampler_t samplerIn =
    CLK_NORMALIZED_COORDS_FALSE |
    CLK_ADDRESS_NONE |
    CLK_FILTER_NEAREST;

__kernel void resizeImage(
    read_only  image2d_t sourceImage,
    write_only image2d_t targetImage
) {
    int inX = get_global_id(0);
    int inY = get_global_id(1);
    int2 posIn = {inX, inY};

    int2 target_dim = get_image_dim(targetImage);
    int2 source_dim = get_image_dim(sourceImage);

    int outX = ((double)inX + 0.4995f) * ((double) target_dim.x / (double) source_dim.x);
    int outY = ((double)inY + 0.4995f) * ((double) target_dim.y / (double) source_dim.y);
    int2 posOut = {outX, outY};

    float4 pixel = read_imagef(sourceImage, samplerIn, posIn);
    write_imagef(targetImage, posOut, pixel);
}
