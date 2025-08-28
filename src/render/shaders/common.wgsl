
#define_import_path common

const PI = radians(180.0);

struct CanvasGlobals {
    screen_size: vec2f,
};

@group(0) @binding(0) var<uniform> GLOBALS: CanvasGlobals;

struct VertexInput {
    @location(0) pos: vec2f,
    @location(1) color: vec4f,
    @location(2) uv: vec2f,
    @location(3) text_uv: vec2f,
};
struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
    @location(1) uv: vec2f,
    @location(2) text_uv: vec2f,
};


fn vs_main_common(
    vertex_idx: u32,
    vertex: VertexInput,
    uv: vec2f,
) -> VertexOutput {
    var out: VertexOutput;

    out.pos = vec4f(vertex.pos / GLOBALS.screen_size * 2.0, 0.0, 1.0);
    out.color = vertex.color;
    out.uv = uv;
    out.text_uv = vertex.text_uv;

    return out;
}