param([string]$Destination)
$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
if (!$Destination) { $Destination = Join-Path $project 'release' }
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Copy-Item (Join-Path $project 'target\i686-pc-windows-gnu\release\StarCraft-HD.exe') $destination
Copy-Item (Join-Path $project 'target\i686-pc-windows-gnu\release\sc_resolution.dll') $destination
Copy-Item (Join-Path $project 'target\i686-pc-windows-gnu\release\version.dll') $destination
Copy-Item (Join-Path $project 'assets\ddraw.dll') $destination
Copy-Item (Join-Path $project 'README.md') (Join-Path $destination 'README.md')
Copy-Item (Join-Path $project 'THIRD-PARTY.txt') (Join-Path $destination 'LICENCIAS.txt')
New-Item -ItemType Directory -Force (Join-Path $destination 'filters') | Out-Null
Copy-Item (Join-Path $project 'assets\filters\*.glsl') (Join-Path $destination 'filters') -Force
New-Item -ItemType Directory -Force (Join-Path $destination 'images') | Out-Null
Copy-Item (Join-Path $project 'images\*.png') (Join-Path $destination 'images') -Force
Write-Output $destination
