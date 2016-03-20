uniform sampler2DArray textures;

in vec4 vColor;
in vec4 vTextureInfo;
in vec2 vTextureOffset;
in float vAtlas;

out vec4 fragColor;

#include lookup_texture

void main() {
    vec4 col = atlasTexture();
    col *= vColor;
    if (col.a == 0.0) discard;
    fragColor = col;
}