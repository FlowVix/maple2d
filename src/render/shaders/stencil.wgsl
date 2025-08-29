
#import common::{
    VertexInput,
    VertexOutput,
    vs_main_common,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    vertex: VertexInput,
) -> VertexOutput {
    return vs_main_common(vertex_idx, vertex, vertex.uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4(1.0);
}