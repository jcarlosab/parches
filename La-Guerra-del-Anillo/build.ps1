$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo build --release --offline --target i686-pc-windows-gnu
    if ($LASTEXITCODE) { throw 'Error de compilación' }
    New-Item -ItemType Directory -Force release | Out-Null
    Copy-Item target/i686-pc-windows-gnu/release/d3d9.dll release/
    Copy-Item README.md release/
    Copy-Item WotR-HD.ini release/
    Copy-Item LICENSE release/
} finally { Pop-Location }
