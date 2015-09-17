const float invAtlasSize = 1.0 / 1024;
vec4 atlasTexture() {
	vec2 tPos = vTextureOffset;
	tPos = mod(tPos, vTextureInfo.zw);
	tPos += vTextureInfo.xy;
	tPos *= invAtlasSize;
	return texture(textures, vec3(tPos, vAtlas));
}	