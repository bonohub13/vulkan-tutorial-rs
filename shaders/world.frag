#version 450

layout(location = 2) in vec3 fragNormalWorld;

layout(location = 0) out vec4 outColor;

const vec3 BACKGROUND_COLOR = vec3(0.5, 0.7, 1.0);
const vec3 WHITE = vec3(1.0, 1.0, 1.0);

vec3 unit_vector(vec3 v) {
    return v / dot(v, v);
}

void main() {
    vec3 unit_direction = unit_vector(fragNormalWorld);
    float a = 0.9 * (1.0 - unit_direction.y);
    vec3 color = (1.0 - a) * WHITE + a * BACKGROUND_COLOR;

    outColor = vec4(color, 1.0);
}
