uniform sampler2D taccum;
uniform sampler2D trevealage;
uniform sampler2DMS tcolor;

uniform int samples;

out vec4 fragColor;

void main() {
    ivec2 C = ivec2(gl_FragCoord.xy);
    vec4 accum = texelFetch(taccum, C, 0);
    float aa = texelFetch(trevealage, C, 0).r;
    vec4 col = texelFetch(tcolor, C, 0);

    for (int i = 1; i < samples; i++) {
        col += texelFetch(tcolor, C, i);
    }
    col /= float(samples);

    float r = accum.a;
    accum.a = aa;
    if (r >= 1.0) {
        fragColor = vec4(col.rgb, 0.0);
    } else {
        vec3 alp = clamp(accum.rgb / clamp(accum.a, 1e-4, 5e4), 0.0, 1.0);
        fragColor = vec4(col.rgb * r  + alp * (1.0 - r), 0.0);
    }
}
