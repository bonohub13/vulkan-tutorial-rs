#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragPosWorld;
layout(location = 2) in vec3 fragNormalWorld;

layout(location = 0) out vec4 outColor;

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
    vec3 diffuseLight = ubo.ambient_light_color.xyz * ubo.ambient_light_color.w;
    vec3 surface_normal = normalize(fragNormalWorld);

    for (int i = 0; i < ubo.num_lights; i++) {
        PointLight light = ubo.point_lights[i];
        vec3 directionToLight = light.position.xyz - fragPosWorld;
        float attenuation = 1.0 / dot(directionToLight, directionToLight);
        float cosAngIncidence = max(dot(surface_normal, normalize(directionToLight)), 0);
        vec3 intensity = light.color.xyz * light.color.w * attenuation;

        diffuseLight += intensity * cosAngIncidence;
    }

    outColor = vec4(diffuseLight * fragColor, 1.0);
}
