$ErrorActionPreference = "Stop"
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path $cargo)) {
    $cargo = "cargo"
}
& $cargo +stable-x86_64-pc-windows-msvc build --release
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}
Copy-Item -Recurse -Force .\可修改文本 .\target\release\可修改文本
Copy-Item -Recurse -Force .\scripts .\target\release\scripts
Copy-Item -Force .\版本更新记录.txt .\target\release\版本更新记录.txt
Write-Host ""
Write-Host "Build complete: target\release\resident_typer_demo.exe"
