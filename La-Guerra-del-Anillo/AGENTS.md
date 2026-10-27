# Maintenance

- Target only the Spanish Rings.exe 1.00.0010, SHA256 cd06fe5b9fd082ecb7856af8035f9fecbc2a7abd16363478ce37707a9950b3e3 (3,481,600 bytes).
- This is an x86 Rust d3d9 proxy. Forward Direct3DCreate9 / Ex to the explicit Windows system path. Validate the file and every code signature before changing memory. Do not patch the game executable on disk.
- Resolution hooks have custom register ABIs, not ordinary C arguments. See the executable machine-code regression tests. Preserve special window modes and all original resolutions.
- Install resolution hooks before the game's INI parser runs; the first Direct3DCreate9 call is too late to recognize saved new modes.
- Camera changes must preserve integer registers, EFLAGS and the complete x87/SSE environment. No Rust floating-point formatting or logging inside per-frame hooks. MenuScene.dll and letterboxed camera views retain the original angle.
- Campaign crash 0xc0000005 at Rings.exe RVA 0x19dec7 was traced to the campaign selector pulse at RVA 0x870ae. It uses absolute GetTickCount * 0.00418879045 as a trig input. After roughly 10.36 days uptime, a signed trig-table index overflows. Capture qa/crash-context.json documents an index 0x997f0000 after 12.42 days uptime. This is independent of the resolution and camera hooks.
- Fix ONLY that pulse's GetTickCount call at RVA 0x870b5 by returning tick % 1500. This is a CALL replacement (E8 plus NOP), not JMP. Do not globally wrap game timing or clamp the shared trig functions.
- Run cargo test --release --offline --target i686-pc-windows-gnu and build.ps1. Current checks execute generated x86 code for resolution round trips, camera decisions with a live x87 stack, and animation timing around overflow/wraparound.
- Verify campaign selection and mission loading in the real game. A launcher/intro smoke check alone does not cover campaign UI.
- release contains only the proxy DLL, WotR-HD.ini and README.md. Do not distribute game files, saves, logs, crash diagnostics or local toolchains.
