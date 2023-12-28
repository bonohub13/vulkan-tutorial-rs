#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragPosWorld;
layout(location = 2) out vec3 fragNormalWorld;

struct PointLight {
    vec4 position;
    vec4 color;
};

layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 projection;
    mat4 view;
    vec4 ambient_light_color;
    PointLight point_lights[10];
    int num_lights;
} ubo;

layout(push_constant) uniform Push {
    mat4 model_matrix;
    mat4 normal_matrix;
} push;

void main() {
    vec4 positionWorld = push.model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection * ubo.view * positionWorld;
    fragNormalWorld = normalize(mat3(push.normal_matrix) * normal);
    fragPosWorld = positionWorld.xyz;
    fragColor = color;
}
