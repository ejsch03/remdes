#version 460 core

// INPUTS
layout(location = 0) in vec2 TexCoord;

// OUTPUTS
layout(location = 0) out vec4 FragColor;

// UNIFORMS
layout(binding = 0) uniform sampler2D uTexture;

void main() {
    FragColor = texture(uTexture, TexCoord);
}
