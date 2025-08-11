fn textureSampleBicubic(tex: texture_2d<f32>, tex_sampler: sampler, texCoords_original: vec2<f32>) -> vec4<f32> {
    var texture_size = vec2<f32>(textureDimensions(tex).xy);

    var invTexSize = 1.0 / texture_size;
   
    var texCoords = texCoords_original * texture_size - 0.5;

    var fxy = fract(texCoords);
    texCoords = texCoords - fxy;

    var xcubic = cubic(fxy.x);
    var ycubic = cubic(fxy.y);

    var c = texCoords.xxyy + vec2(-0.5, 1.5).xyxy;
    
    var s = vec4(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
    var offset = c + vec4(xcubic.yw, ycubic.yw) / s;
    
    offset = offset * invTexSize.xxyy;
    
    var sample0 = textureSample(tex, tex_sampler, offset.xz);
    var sample1 = textureSample(tex, tex_sampler, offset.yz);
    var sample2 = textureSample(tex, tex_sampler, offset.xw);
    var sample3 = textureSample(tex, tex_sampler, offset.yw);

    var sx = s.x / (s.x + s.y);
    var sy = s.z / (s.z + s.w);

    return mix(mix(sample3, sample2, vec4(sx)), mix(sample1, sample0, vec4(sx)), vec4(sy));
}
