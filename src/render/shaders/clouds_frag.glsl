in vec4 fColor;
in vec3 fLighting;

out vec4 fragColor;

void main() {
	vec4 col = fColor;
	col.rgb *= fLighting;
	fragColor = col;
}
