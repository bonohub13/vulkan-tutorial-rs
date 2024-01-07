#version 450

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler2D material;

vec2 unit_vector(vec2 v);

void main() {
    vec4 computed_color = texture(material, fragTexCoord);

    if (computed_color.w == 0.0) {
        float a = 0.75 * (fragTexCoord.y + 1.0);

        outColor = vec4((a - 1.0) * vec3(1.0) + a * vec3(0.5, 0.7, 1.0), 1.0);
    } else {
        outColor = computed_color;
    }
}

vec2 unit_vector(vec2 v) {
    float length = sqrt(dot(v, v));

    return v / length;
}
