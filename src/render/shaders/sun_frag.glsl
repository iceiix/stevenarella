uniform sampler2DArray textures;
uniform vec4 colorMul[10];

in vec4 vTextureInfo;
in vec2 vTextureOffset;
in float vAtlas;
in float vID;

out vec4 fragColor;

#include lookup_texture

void main() {
	vec4 col = atlasTexture();
	if (col.a <= 0.05) discard;
	fragColor = col * colorMul[int(vID)];
}
