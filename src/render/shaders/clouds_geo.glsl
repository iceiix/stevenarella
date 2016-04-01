layout(points) in;
layout(triangle_strip, max_vertices = 24) out;

uniform mat4 perspectiveMatrix;
uniform mat4 cameraMatrix;
uniform vec3 offset;
uniform float cloudOffset;

uniform vec4 textureInfo;
uniform float atlas;
uniform sampler2DArray textures;
uniform sampler2D cloudMap;

in vec3 vLighting[];

out vec3 fLighting;
out vec4 fColor;

void setVertex(vec3 base, vec3 off, float color) {
	gl_Position = perspectiveMatrix * cameraMatrix * vec4(base + off*vec3(1.0,-1.0,1.0), 1.0);
	fColor = vec4(color, color, color, 1.0);
	fLighting = vLighting[0];
	EmitVertex();
}

float coffset = cloudOffset;
const float invAtlasSize = 1.0 / 1024.0;
vec4 atlasTexture(vec2 tPos) {
	tPos.y += floor(coffset);
	tPos = mod(tPos, textureInfo.zw);
	tPos += textureInfo.xy;
	tPos *= invAtlasSize;
	return texture(textures, vec3(tPos, atlas));
}

ivec2 texP, heightP;

bool isSolid(ivec2 pos) {
	float height = texelFetch(cloudMap, ivec2(mod(heightP + pos, 512)), 0).r;
	if (height >= 127.0/255.0) return false;
	return atlasTexture(vec2(texP + pos)).r + height > (250.0 / 255.0);
}

bool isFutureSolid(ivec2 pos) {
	// Sneak a peak into the future
	coffset += 1.0;
	bool ret = isSolid(pos);
	coffset -= 1.0;
	return ret;
}

void main() {
	vec3 base = floor(offset) + gl_in[0].gl_Position.xyz;
	texP = ivec2(gl_in[0].gl_Position.xz + 160.0 + offset.xz) - ivec2(0.0, -1.0);
	heightP = ivec2(mod(base.xz, 512));
	if (!isSolid(ivec2(0))) return;

	float backOffset = 1.0 - fract(cloudOffset);
	float frontOffset = -fract(cloudOffset);
	if (!isFutureSolid(ivec2(0, -1))) {
		frontOffset = 0.0;
	}

	// Top
	setVertex(base, vec3(0.0, 1.0, frontOffset), 1.0);
	setVertex(base, vec3(1.0, 1.0, frontOffset), 1.0);
	setVertex(base, vec3(0.0, 1.0, backOffset), 1.0);
	setVertex(base, vec3(1.0, 1.0, backOffset), 1.0);
	EndPrimitive();

	// Bottom
	setVertex(base, vec3(0.0, 0.0, frontOffset), 0.7);
	setVertex(base, vec3(0.0, 0.0, backOffset), 0.7);
	setVertex(base, vec3(1.0, 0.0, frontOffset), 0.7);
	setVertex(base, vec3(1.0, 0.0, backOffset), 0.7);
	EndPrimitive();

	if (!isSolid(ivec2(-1, 0)) || !isFutureSolid(ivec2(-1, -1))) {
		float sideOffset = backOffset;
		if (isSolid(ivec2(-1, 1)) && !isFutureSolid(ivec2(-1, 0))) {
			sideOffset = 1.0;
		}
		// -X
		setVertex(base, vec3(0.0, 0.0, frontOffset), 0.8);
		setVertex(base, vec3(0.0, 1.0, frontOffset), 0.8);
		setVertex(base, vec3(0.0, 0.0, sideOffset), 0.8);
		setVertex(base, vec3(0.0, 1.0, sideOffset), 0.8);
		EndPrimitive();
	}

	if (!isSolid(ivec2(1, 0)) || !isFutureSolid(ivec2(1, -1))) {
		float sideOffset = backOffset;
		if (isSolid(ivec2(1, 1)) && !isFutureSolid(ivec2(1, 0))) {
			sideOffset = 1.0;
		}
		// +X
		setVertex(base, vec3(1.0, 0.0, frontOffset), 0.8);
		setVertex(base, vec3(1.0, 0.0, sideOffset), 0.8);
		setVertex(base, vec3(1.0, 1.0, frontOffset), 0.8);
		setVertex(base, vec3(1.0, 1.0, sideOffset), 0.8);
		EndPrimitive();
	}

	if (!isSolid(ivec2(0, 1)) || !isFutureSolid(ivec2(0, 0))) {
		// -Z
		setVertex(base, vec3(0.0, 0.0, backOffset), 0.8);
		setVertex(base, vec3(0.0, 1.0, backOffset), 0.8);
		setVertex(base, vec3(1.0, 0.0, backOffset), 0.8);
		setVertex(base, vec3(1.0, 1.0, backOffset), 0.8);
		EndPrimitive();
	}

	if (!isSolid(ivec2(0, -1)) || !isFutureSolid(ivec2(0, -2))) {
		// +Z
		setVertex(base, vec3(0.0, 0.0, frontOffset), 0.8);
		setVertex(base, vec3(1.0, 0.0, frontOffset), 0.8);
		setVertex(base, vec3(0.0, 1.0, frontOffset), 0.8);
		setVertex(base, vec3(1.0, 1.0, frontOffset), 0.8);
		EndPrimitive();
	}
}
