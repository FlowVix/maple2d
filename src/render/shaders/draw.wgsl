
#import common::{
    VertexInput,
    VertexOutput,
    Globals,
    GLOBALS,
    vs_main_common,
    PI,
}

@group(1) @binding(0) var TEXTURE_T: texture_2d<f32>;
@group(1) @binding(1) var TEXTURE_S: sampler;

@group(2) @binding(0) var TEXT_MASK_T: texture_2d<f32>;
@group(2) @binding(1) var TEXT_MASK_S: sampler;
@group(2) @binding(2) var TEXT_COLOR_T: texture_2d<f32>;
@group(2) @binding(3) var TEXT_COLOR_S: sampler;


@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    vertex: VertexInput,
) -> VertexOutput {
    return vs_main_common(vertex_idx, vertex, vertex.uv / vec2f(textureDimensions(TEXTURE_T)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var out = in.color;
    if in.uv.x >= 0.0 {
        out *= textureSample(TEXTURE_T, TEXTURE_S, in.uv);
    }
    if in.text_uv.x >= 0.0 {
        if in.text_uv.y >= 0.0 {
            out.a *= sqrt(textureSample(
                TEXT_MASK_T,
                TEXT_MASK_S,
                in.text_uv / vec2f(textureDimensions(TEXT_MASK_T)),
            ).r);
        } else {
            out *= textureSample(
                TEXT_COLOR_T,
                TEXT_COLOR_S,
                in.text_uv / vec2f(textureDimensions(TEXT_COLOR_T)) + vec2f(0.0, 2.0)
            );
        }
    }
    return out;
}