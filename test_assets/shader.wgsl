[[group(0), binding(0)]]
var view_proj: mat4x4<f32>;

[[group(1), binding(0)]]
var world: mat4x4<f32>;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec3<f32>,
    [[location(2)]] uv: vec2<f32>,
) -> VetexOutput {
    var out: VertexOutput;
    out.color = color;
    out.uv = uv;
    out.position = view_proj * world * vec4<f32>(position, 1.0);
    return output;
}

[[group(2), binding(0)]]
var t_diffuse: texture_2d<f32>;

[[group(2), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var c: vec4<f32> = textureSample(t_diffuse, s_siffuse, in.uv);
    return c * vec4<f32>(in.color, 1.0);
}