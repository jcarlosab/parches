param([string]$GameExe = $env:STARCRAFT_EXE)
$ErrorActionPreference = 'Stop'
$project = $PSScriptRoot
if (Test-Path (Join-Path $project 'toolchain\cargo\bin\cargo.exe')) {
    $env:RUSTUP_HOME = Join-Path $project 'toolchain\rustup'
    $env:CARGO_HOME = Join-Path $project 'toolchain\cargo'
    $env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
    $env:PYTHONPATH = Join-Path $project 'toolchain\pylib'
}
if (!$GameExe) {
    $candidate = Get-Item $project
    while ($candidate -and !$GameExe) {
        $exe = Join-Path $candidate.FullName 'StarCraft.exe'
        if (Test-Path -LiteralPath $exe) { $GameExe = $exe }
        $candidate = $candidate.Parent
    }
}
Push-Location $project
try {
    if ($GameExe) {
        python tools/generate_tables.py --exe $GameExe
        if ($LASTEXITCODE) { throw 'No se pudieron verificar las tablas.' }
    }
    if (Test-Path (Join-Path $project 'toolchain\cargo\bin\cargo.exe')) {
        cargo build --release --offline --target i686-pc-windows-gnu
    } else {
        cargo +stable-i686-pc-windows-gnu build --release --offline --target i686-pc-windows-gnu
    }
    if ($LASTEXITCODE) { throw 'La compilación ha fallado.' }
    $built = Join-Path $project 'target\i686-pc-windows-gnu\release'
    python tools/embed_icon.py (Join-Path $built 'StarCraft-HD.exe')
    if ($LASTEXITCODE) { throw 'No se pudo incrustar el icono.' }
    New-Item -ItemType Directory -Force release/filters | Out-Null
    Copy-Item (Join-Path $built 'StarCraft-HD.exe') release/
    Copy-Item (Join-Path $built 'sc_resolution.dll') release/
    Copy-Item (Join-Path $built 'version.dll') release/
    Copy-Item assets/ddraw.dll release/
    Copy-Item assets/filters/*.glsl release/filters/ -Force
    Copy-Item THIRD-PARTY.txt release/LICENCIAS.txt
    Copy-Item README.md release/README.md
    Copy-Item images release/ -Recurse -Force
    if ($GameExe) {
        $check = Start-Process -FilePath (Join-Path $project 'release\StarCraft-HD.exe') -ArgumentList '--self-test','--game',('"'+(Split-Path $GameExe -Parent)+'"') -WindowStyle Hidden -Wait -PassThru
        if ($check.ExitCode) { throw 'La comprobación ha fallado.' }
    }
} finally { Pop-Location }
