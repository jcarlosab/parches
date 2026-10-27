"""Check the x86 loader ABI and absence of remote injection APIs in the launcher."""
from pathlib import Path
import re
import sys
import pefile

root = Path(__file__).resolve().parents[1]
folder = Path(sys.argv[1]) if len(sys.argv) > 1 else root / "release"
launcher = pefile.PE(str(folder / "StarCraft-HD.exe"))
imports = {i.name for d in launcher.DIRECTORY_ENTRY_IMPORT for i in d.imports}
forbidden = {b"WriteProcessMemory", b"ReadProcessMemory", b"VirtualAllocEx",
             b"CreateRemoteThread", b"NtCreateThreadEx", b"QueueUserAPC"}
assert not imports & forbidden, imports & forbidden
proxy = pefile.PE(str(folder / "version.dll"))
assert proxy.FILE_HEADER.Machine == 0x14c
assert proxy.OPTIONAL_HEADER.ImageBase == 0x61000000
expected = {(int(ordinal), name.encode()) for name, ordinal in re.findall(
    r"^\s+(\w+)=forward_\d+ @(\d+)",
    (root / "bootstrap/exports.def").read_text(), re.M)}
actual = {(s.ordinal, s.name) for s in proxy.DIRECTORY_ENTRY_EXPORT.symbols}
assert len(expected) == 17 and expected <= actual
print("OK: 17 Version exports retain their ordinals; x86 base; no remote injection APIs")
