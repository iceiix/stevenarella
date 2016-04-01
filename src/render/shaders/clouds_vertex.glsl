in vec3 aPosition;

uniform float lightLevel;
uniform float skyOffset;

out vec3 vLighting;

#include get_light

void main() {
	vec3 pos = vec3(aPosition.x, -aPosition.y, aPosition.z);
	gl_Position = vec4(pos, 1.0);

	vLighting = getLight(vec2(0.0, 15.0));
}
