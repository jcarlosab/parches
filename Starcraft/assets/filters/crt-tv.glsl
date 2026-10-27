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
    vec3 blur=(texture(Texture,uv-vec2(pixel.x,0.0)).rgb+texture(Texture,uv+vec2(pixel.x,0.0)).rgb)*0.5;
    c=mix(c,blur,0.24);
    float beam=0.87+0.13*cos(screen.y*480.0*6.2831853);
    float stripe=mod(floor(screen.x*OutputSize.x),3.0);
    vec3 phosphor=stripe<1.0?vec3(1.0,0.95,0.95):(stripe<2.0?vec3(0.95,1.0,0.95):vec3(0.95,0.95,1.0));
    float vignette=1.0-0.09*dot(screen-0.5,screen-0.5);
    c=pow(max(c,vec3(0.0)),vec3(0.94))*beam*phosphor*vec3(1.06,1.025,0.98)*vignette;
    FragColor=vec4(clamp(c,0.0,1.0),1.0);
}
#endif
