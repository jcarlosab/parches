"""Read-only verification of all three original console images; QA files aren't shipped."""
from pathlib import Path
from io import BytesIO
import struct
import zlib
import mpyq
import dclimplode
from PIL import Image

root = Path(__file__).resolve().parents[1]
archive = mpyq.MPQArchive(str(root.parent / "StarDat.mpq"), listfile=False)
output = root / "qa"
output.mkdir(exist_ok=True)
for race, label in [("t", "Terran"),("z", "Zerg"),("p", "Protoss")]:
    filename = "game\\" + race + "console.pcx"
    entry = archive.block_table[archive.get_hash_table_entry(filename).block_table_index]
    archive.file.seek(entry.offset + archive.header["offset"])
    raw = archive.file.read(entry.archived_size)
    key = ((archive._hash(filename.split("\\")[-1],"TABLE")+entry.offset)^entry.size)&0xffffffff
    sector_size = 512 << archive.header["sector_size_shift"]
    sectors = (entry.size+sector_size-1)//sector_size
    positions = struct.unpack("<%dI"%(sectors+1),archive._decrypt(raw[:4*(sectors+1)],(key-1)&0xffffffff))
    decoded = bytearray()
    for i in range(sectors):
        block=raw[positions[i]:positions[i+1]]
        length=len(block)//4*4
        block=archive._decrypt(block[:length],(key+i)&0xffffffff)+block[length:]
        expected=min(sector_size,entry.size-len(decoded))
        if len(block)<expected:
            if block[0]==8: block=dclimplode.decompressobj().decompress(block[1:])
            elif block[0]==2: block=zlib.decompress(block[1:])
            else: raise ValueError("Unsupported compression")
        assert len(block)==expected
        decoded+=block
    assert len(decoded)==entry.size
    image=Image.open(BytesIO(decoded)); assert image.mode=="P" and image.size==(640,480)
    pixels=image.tobytes()
    first=min(y for y in range(480) if any(pixels[y*640:(y+1)*640]))
    print(f"{label}: 640x480, first opaque row {first}, {sum(p!=0 for p in pixels)} opaque pixels")
    image.convert("RGB").save(output/(race+"console.png"))
    (output/(race+"console.raw")).write_bytes(pixels)
