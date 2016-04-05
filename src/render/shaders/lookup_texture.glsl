const float invAtlasSize = 1.0 / 1024;
vec4 atlasTexture() {
    vec2 tPos = vTextureOffset;
    tPos = clamp(tPos, vec2(0.1), vTextureInfo.zw - 0.1);
    tPos += vTextureInfo.xy;
    tPos *= invAtlasSize;
    return texture(textures, vec3(tPos, vAtlas));
}
