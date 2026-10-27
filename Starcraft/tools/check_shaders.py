"""Compile and link our filters against the installed OpenGL driver, offscreen."""
import ctypes as c
from pathlib import Path
import struct

u=c.WinDLL('user32'); g=c.WinDLL('gdi32'); gl=c.WinDLL('opengl32')
u.CreateWindowExW.argtypes=[c.c_uint,c.c_wchar_p,c.c_wchar_p,c.c_uint,c.c_int,c.c_int,c.c_int,c.c_int,c.c_void_p,c.c_void_p,c.c_void_p,c.c_void_p]
u.CreateWindowExW.restype=c.c_void_p
u.GetDC.argtypes=[c.c_void_p];u.GetDC.restype=c.c_void_p
u.ReleaseDC.argtypes=[c.c_void_p,c.c_void_p];u.DestroyWindow.argtypes=[c.c_void_p]
g.ChoosePixelFormat.argtypes=[c.c_void_p,c.c_void_p]
g.SetPixelFormat.argtypes=[c.c_void_p,c.c_int,c.c_void_p]
gl.wglCreateContext.argtypes=[c.c_void_p];gl.wglCreateContext.restype=c.c_void_p
gl.wglMakeCurrent.argtypes=[c.c_void_p,c.c_void_p]
gl.wglDeleteContext.argtypes=[c.c_void_p]
gl.wglGetProcAddress.argtypes=[c.c_char_p];gl.wglGetProcAddress.restype=c.c_void_p
gl.glGetString.argtypes=[c.c_uint];gl.glGetString.restype=c.c_char_p

def proc(name, result, *args):
    addr=gl.wglGetProcAddress(name.encode())
    if addr in (None,1,2,3,c.c_void_p(-1).value):raise RuntimeError(name+' unavailable')
    return c.WINFUNCTYPE(result,*args)(addr)

window=u.CreateWindowExW(0,'STATIC','Shader validation',0,0,0,16,16,None,None,None,None)
dc=u.GetDC(window);context=None
try:
    pfd=c.create_string_buffer(40)
    struct.pack_into('<HHI',pfd,0,40,1,0x25)
    pfd[9]=32
    pf=g.ChoosePixelFormat(dc,pfd)
    assert pf and g.SetPixelFormat(dc,pf,pfd)
    context=gl.wglCreateContext(dc)
    assert context and gl.wglMakeCurrent(dc,context)
    print('OpenGL:',gl.glGetString(0x1f02).decode())
    create=proc('glCreateShader',c.c_uint,c.c_uint)
    source=proc('glShaderSource',None,c.c_uint,c.c_int,c.POINTER(c.c_char_p),c.c_void_p)
    compile_=proc('glCompileShader',None,c.c_uint)
    status=proc('glGetShaderiv',None,c.c_uint,c.c_uint,c.POINTER(c.c_int))
    info=proc('glGetShaderInfoLog',None,c.c_uint,c.c_int,c.c_void_p,c.c_char_p)
    program=proc('glCreateProgram',c.c_uint)
    attach=proc('glAttachShader',None,c.c_uint,c.c_uint)
    link=proc('glLinkProgram',None,c.c_uint)
    pstatus=proc('glGetProgramiv',None,c.c_uint,c.c_uint,c.POINTER(c.c_int))
    pinfo=proc('glGetProgramInfoLog',None,c.c_uint,c.c_int,c.c_void_p,c.c_char_p)
    delete=proc('glDeleteShader',None,c.c_uint)
    pdelete=proc('glDeleteProgram',None,c.c_uint)
    for file in sorted((Path(__file__).resolve().parents[1]/'assets/filters').glob('*.glsl')):
        for version in [130,150]:
            shaders=[];pr=program()
            try:
                for stage,kind in [('VERTEX',0x8b31),('FRAGMENT',0x8b30)]:
                    text=f'#version {version}\n#define {stage}\n'+file.read_text().split('\n',1)[1]
                    shader=create(kind);shaders.append(shader)
                    source(shader,1,c.byref(c.c_char_p(text.encode())),None);compile_(shader)
                    ok=c.c_int();status(shader,0x8b81,c.byref(ok))
                    buf=c.create_string_buffer(8192);info(shader,len(buf),None,buf)
                    assert ok.value,(file.name,stage,buf.value)
                    attach(pr,shader)
                link(pr);ok=c.c_int();pstatus(pr,0x8b82,c.byref(ok))
                buf=c.create_string_buffer(8192);pinfo(pr,len(buf),None,buf)
                assert ok.value,(file.name,buf.value)
                print(file.name,version,'compile + link OK')
            finally:
                pdelete(pr)
                for shader in shaders:delete(shader)
finally:
    gl.wglMakeCurrent(None,None)
    if context:gl.wglDeleteContext(context)
    if dc:u.ReleaseDC(window,dc)
    if window:u.DestroyWindow(window)
