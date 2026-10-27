"""Convert the generated pixel-art PNG to ICO and embed native PE icon resources."""
from pathlib import Path
import ctypes as C
import struct
import sys
from io import BytesIO
from PIL import Image

root = Path(__file__).resolve().parents[1]
png = root / "assets/zerg-icon.png"
ico = root / "assets/zerg-icon.ico"
image = Image.open(png).convert("RGBA")
sizes=[16,24,32,48,64,128,256]
directory=bytearray(struct.pack("<HHH",0,1,len(sizes)))
payload=bytearray()
for size in sizes:
    frame=BytesIO()
    image.resize((size,size),Image.Resampling.NEAREST).save(frame,format="PNG")
    blob=frame.getvalue()
    directory+=struct.pack("<BBBBHHII",size%256,size%256,0,0,1,32,len(blob),6+16*len(sizes)+len(payload))
    payload+=blob
ico.write_bytes(directory+payload)
exe = Path(sys.argv[1]).resolve()
data = ico.read_bytes()
reserved, kind, count = struct.unpack_from("<HHH", data)
kernel = C.WinDLL("kernel32", use_last_error=True)
kernel.BeginUpdateResourceW.argtypes = [C.c_wchar_p,C.c_int]
kernel.BeginUpdateResourceW.restype = C.c_void_p
kernel.UpdateResourceW.argtypes = [C.c_void_p,C.c_void_p,C.c_void_p,C.c_ushort,C.c_void_p,C.c_uint]
kernel.UpdateResourceW.restype = C.c_int
kernel.EndUpdateResourceW.argtypes = [C.c_void_p,C.c_int]
kernel.EndUpdateResourceW.restype = C.c_int
handle = kernel.BeginUpdateResourceW(str(exe),False)
if not handle:
    raise C.WinError(C.get_last_error())
try:
    group = bytearray(struct.pack("<HHH", reserved,kind,count))
    for i in range(count):
        width,height,colors,zero,planes,bpp,size,offset=struct.unpack_from("<BBBBHHII",data,6+i*16)
        blob=C.create_string_buffer(data[offset:offset+size])
        if not kernel.UpdateResourceW(handle,3,i+1,0,blob,size):
            raise C.WinError(C.get_last_error())
        group += struct.pack("<BBBBHHIH",width,height,colors,zero,planes,bpp,size,i+1)
    blob=C.create_string_buffer(bytes(group))
    if not kernel.UpdateResourceW(handle,14,1,0,blob,len(group)):
        raise C.WinError(C.get_last_error())
except:
    kernel.EndUpdateResourceW(handle,True)
    raise
if not kernel.EndUpdateResourceW(handle,False):
    raise C.WinError(C.get_last_error())
print(f"Icono incrustado: {count} tamaños")
