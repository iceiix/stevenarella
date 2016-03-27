in vec3 aPosition;
in vec4 aTextureInfo;
in ivec3 aTextureOffset;
in vec4 aColor;
in int id;

uniform mat4 perspectiveMatrix;
uniform mat4 cameraMatrix;
uniform mat4 modelMatrix[10];
uniform float lightLevel;
uniform float skyOffset;
uniform vec2 lighting;

out vec4 vColor;
out vec4 vTextureInfo;
out vec2 vTextureOffset;
out float vAtlas;
out float vID;
out vec3 vLighting;

#include get_light

void main() {
	vec3 pos = vec3(aPosition.x, -aPosition.y, aPosition.z);
	gl_Position = perspectiveMatrix * cameraMatrix * modelMatrix[id] * vec4(pos, 1.0);

	vColor = aColor;
	vTextureInfo = aTextureInfo;
	vTextureOffset = aTextureOffset.xy / 16.0;
	vAtlas = aTextureOffset.z;
	vID = float(id);

	vLighting = getLight(lighting);
}
