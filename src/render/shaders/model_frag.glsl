uniform sampler2DArray textures;
uniform vec4 colorMul[10];

in vec4 vColor;
in vec4 vTextureInfo;
in vec2 vTextureOffset;
in float vAtlas;
in vec3 vLighting;
in float vID;

out vec4 fragColor;

#include lookup_texture

void main() {
	vec4 col = atlasTexture();
	if (col.a <= 0.05) discard;
	col *= vColor;
	col.rgb *= vLighting;
	fragColor = col * colorMul[int(vID)];
}
