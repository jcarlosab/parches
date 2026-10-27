#version 130
// Original single-pass display filter for this launcher. No geometry distortion.
#ifdef VERTEX
in vec2 VertexCoord;
in vec2 TexCoord;
out vec2 uv;
void main() { gl_Position=vec4(VertexCoord,0.0,1.0); uv=TexCoord; }
#endif
#ifdef FRAGMENT
uniform sampler2D Texture;
uniform vec2 InputSize;
uniform vec2 TextureSize;
uniform vec2 OutputSize;
in vec2 uv;
out vec4 FragColor;
void main() {
    vec2 pixel=1.0/TextureSize;
    vec2 screen=uv*TextureSize/InputSize;
    vec3 c=texture(Texture,uv).rgb;
    float beam=0.94+0.06*cos(screen.y*OutputSize.y*3.14159265);
    float stripe=mod(floor(screen.x*OutputSize.x),3.0);
    vec3 phosphor=stripe<1.0?vec3(1.0,0.94,0.94):(stripe<2.0?vec3(0.94,1.0,0.94):vec3(0.94,0.94,1.0));
    c=pow(max(c,vec3(0.0)),vec3(0.97))*beam*phosphor*1.045;
    FragColor=vec4(clamp(c,0.0,1.0),1.0);
}
#endif
