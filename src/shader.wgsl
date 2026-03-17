struct CameraUniform {
    view_proj: mat4x4<f32>,
};
struct GravityUniform {
    position: vec3<f32>,
    strength: f32,
}

@group(1) @binding(0)
var<uniform> gravity: GravityUniform;

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@vertex
fn vs_main_no_gravity(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3, 
    );
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3, 
    );
    var out: VertexOutput;
    let zx_distance = distance(vec2<f32>(model.position.x ,model.position.z),vec2<f32>(gravity.position.x ,gravity.position.z));
    let gravtiy_distortion = (1.0 / (zx_distance + 1.0)) * gravity.strength;
    let gravity_affected_vectors = vec3<f32>(model.position.x, model.position.y - gravtiy_distortion, model.position.z);
    out.color = mix(vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), gravtiy_distortion);
    out.clip_position = camera.view_proj * model_matrix  * vec4<f32>(gravity_affected_vectors, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
