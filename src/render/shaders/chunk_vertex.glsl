in vec3 aPosition;
in vec4 aTextureInfo;
in vec3 aTextureOffset;
in vec3 aColor;
in vec2 aLighting;

uniform mat4 perspectiveMatrix;
uniform mat4 cameraMatrix;
uniform ivec3 offset;
uniform float lightLevel;
uniform float skyOffset;

out vec3 vColor;
out vec4 vTextureInfo;
out vec2 vTextureOffset;
out float vAtlas;
out vec3 vLighting;

#include get_light

void main() {
    vec3 pos = vec3(aPosition.x, -aPosition.y, aPosition.z);
    vec3 o = vec3(offset.x, -offset.y / 4096.0, offset.z);
    gl_Position = perspectiveMatrix * cameraMatrix * vec4(pos + o * 16.0, 1.0);

    vColor = aColor;
    vTextureInfo = aTextureInfo;
    vTextureOffset = aTextureOffset.xy / 16.0;
    vAtlas = aTextureOffset.z;

    vLighting = getLight(aLighting / (4000.0));
}
