in ivec3 aPosition;
in vec4 aTextureInfo;
in ivec3 aTextureOffset;
in vec4 aColor;

out vec4 vColor;
out vec4 vTextureInfo;
out vec2 vTextureOffset;
out float vAtlas;

uniform vec2 screenSize;

void main() {
    vec2 pos = vec2(aPosition.xy) / screenSize;
    gl_Position = vec4((pos.x-0.5)*2.0, -(pos.y-0.5)*2.0, float(-aPosition.z) / float(0xFFFF-1), 1.0);
    vColor = aColor;
    vTextureInfo = aTextureInfo;
    vTextureOffset = vec2(aTextureOffset.xy) / 16.0;
    vAtlas = float(aTextureOffset.z);
}
