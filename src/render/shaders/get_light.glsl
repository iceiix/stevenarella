
vec3 getLight(vec2 light) {
    vec2 li = pow(vec2(lightLevel), 15.0 - light);
    float skyTint = skyOffset * 0.95 + 0.05;
    float bl = li.x;
    float sk = li.y * skyTint;

    float skyRed = sk * (skyOffset * 0.65 + 0.35);
    float skyGreen = sk * (skyOffset * 0.65 + 0.35);
    float blockGreen = bl * ((bl * 0.6 + 0.4) * 0.6 + 0.4);
    float blockBlue = bl * (bl * bl * 0.6 + 0.4);

    vec3 col = vec3(
        skyRed + bl,
        skyGreen + blockGreen,
        sk + blockBlue
    );

    col = col * 0.96 + 0.03;

    float gamma = 0.0;
    vec3 invCol = 1.0 - col;
    invCol = 1.0 - invCol * invCol * invCol * invCol;
    col = col * (1.0 - gamma) + invCol * gamma;
    col = col * 0.96 + 0.03;

    return clamp(col, 0.0, 1.0);
}
