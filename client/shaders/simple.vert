#version 460 core

// INPUTS
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;

// OUTPUTS
layout(location = 0) out vec2 TexCoord;

void main() {
    TexCoord = vec2(aTexCoord.x, 1.0 - aTexCoord.y);
    gl_Position = vec4(aPos, 0.0, 1.0);
}
