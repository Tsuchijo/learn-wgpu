// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_dir: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.view_dir = normalize(camera.camera_pos.xyz - model.position);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn sdf(in: vec3<f32>) -> f32 {
    var center: vec3<f32> = vec3<f32> (0.0, -1.0, -1.0);
    return distance(in, center) - 0.1;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.clip_position.xyz;
    for(var i = 0; i < 200; i++){
        if(sdf(position) <= 0.1){
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }
        position += in.view_dir * sdf(position);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

 

 