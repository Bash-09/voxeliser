#version 430

in vec3 pos;
in vec3 norm;
in vec2 tc;

out vec3 col;

uniform mat4 pvmat;
uniform mat4 tmat;

void main() {
    col = norm;
    vec4 world_pos = tmat * vec4(pos, 1.0);
    vec4 pos = pvmat * world_pos;
    gl_Position = pos;
}