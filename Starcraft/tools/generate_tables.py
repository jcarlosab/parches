"""Build verified, compact patch data. Requires pefile + capstone.

Instruction map adapted from inwenis/decompile-sc (MIT, see THIRD-PARTY).
Our multiply rewrite removes its three-power-of-two width restriction.
This tool only reads StarCraft.exe; it never edits the executable.
"""
from pathlib import Path
import importlib.util
import struct

ROOT = Path(__file__).resolve().parents[1]
import argparse, os
parser=argparse.ArgumentParser()
parser.add_argument('--exe', default=os.environ.get('STARCRAFT_EXE', str(ROOT.parent / 'StarCraft.exe')))
args=parser.parse_args()
source = ROOT / "tools/vendor/renderer_patch_sites.py"
text = source.read_text(encoding="utf-8")
text = text.replace("CONSOLE_SHIFT_Y = PF_H - STOCK_PF_H", "CONSOLE_SHIFT_Y = H - STOCK_H")
text = text.replace("K_Y = PF_H // 32", "K_Y = (PF_H - 24 + 31) // 32")
start = text.index("    bits = [i for i in range(32)")
end = text.index("    for va, note in [", start)
text = text[:start] + '''    # EAX starts as the source row. Preserve EBX and ECX's original
    # scratch results, but compute EAX directly for any supported pitch.
    b.code(0x0040C275,
           "c1e0098b0da8846200c1e307c1e10503c303c1",
           "69c0" + le32(TERRAIN_PITCH) + "8b0da8846200c1e307c1e10590",
           "terrain.pitch.multiply.rust", 2,
           "direct IMUL replaces the three-shift sum; next ADD defines flags")

''' + text[end:]
module = type(importlib.util)("patch_map")
module.__file__ = str(source)
exec(compile(text, str(source), "exec"), module.__dict__)
out = ROOT / "resolution/data"
out.mkdir(parents=True, exist_ok=True)
for w, h in [(960,540),(1280,720),(1280,800),(1280,880),(1536,864),(1600,900),(1920,1080)]:
    # Terrain now extends behind the compact HUD, including the bottom corners.
    # Console anchors still use H-480, independently from the terrain height.
    b = module.build(module.Image(str(Path(args.exe).resolve())), w, h, h)
    assert b.geom["PF_H"] == h and b.geom["CONSOLE_SHIFT_Y"] == h-480
    if b.errors:
        raise RuntimeError(b.errors)
    # The runtime supplies its own HUD-aware hit test at the original entry.
    b.patches = [p for p in b.patches if p.name != "console.hittest.yshift"]
    mid = 0
    b.data(0x005136CC, module.le32(496), module.le32(496+mid),
           "console.cardrect.left.rust",3,"console anchored left")
    b.data(0x005136D4, module.le32(639), module.le32(639+mid),
           "console.cardrect.right.rust",3,"console anchored left")
    # The minimap control's own rectangle stays root-relative. Its separate
    # world-coordinate converter and absolute screen clip need the same shift.
    b.cave(0x004A3D77,"668b5604662910",
           "668b5604662910" + "668128" + mid.to_bytes(2,"little").hex(),
           "minimap.centre.input.x.rust",3,"subtract root x before minimap-to-world conversion")
    b.cave(0x004A452B,"0fbf4804890d002d5100",
           "0fbf4804" + "81c1" + module.le32(mid) + "890d002d5100",
           "minimap.centre.clip.left.rust",3,"absolute minimap left includes root x")
    b.cave(0x004A4550,"8d4c11ff890d082d5100",
           "8d4c11ff" + "81c1" + module.le32(mid) + "890d082d5100",
           "minimap.centre.clip.right.rust",3,"absolute minimap right includes root x")
    if b.errors:
        raise RuntimeError(b.errors)
    patches = [p for p in b.patches if not p.noop]
    data = bytearray(b"SCR1" + struct.pack("<HHII", w, h, b.geom["GRID_BYTES"], len(patches)))
    for p in patches:
        data += struct.pack("<IBBBI", p.va, len(p.expect), 255 if p.fixup_off is None else p.fixup_off,
                            len(p.cave or b""), p.fixup_addend)
        data += p.expect + p.patch + (p.cave or b"")
    (out / f"{w}x{h}.bin").write_bytes(data)
    print(f"{w}x{h}: {len(patches)} verified sites, {len(data)} bytes")
