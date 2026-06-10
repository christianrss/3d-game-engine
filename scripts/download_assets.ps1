# Baixa texturas PBR CC0 (Poly Haven) e verifica modelos glTF locais.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$TexDir = Join-Path $Root "assets\textures"
$RockDir = Join-Path $TexDir "rock"
$ModelsDir = Join-Path $Root "assets\models"

New-Item -ItemType Directory -Force -Path $TexDir, $RockDir, $ModelsDir | Out-Null

function Download-File($Url, $Dest) {
    if (Test-Path $Dest) {
        Write-Host "OK (exists): $Dest"
        return
    }
    Write-Host "Downloading: $Url"
    Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
}

# Areia — desert sand (1k JPG)
$SandBase = "https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/desert_sand/desert_sand"
Download-File "$SandBase/diff_1k.jpg" (Join-Path $TexDir "sand_diff.jpg")
Download-File "$SandBase/nor_gl_1k.jpg" (Join-Path $TexDir "sand_normal.jpg")
Download-File "$SandBase/rough_1k.jpg" (Join-Path $TexDir "sand_rough.jpg")

# Rocha — cliff rock (1k JPG)
$RockBase = "https://dl.polyhaven.org/file/ph-assets/Textures/jpg/1k/cliff_side/cliff_side"
Download-File "$RockBase/diff_1k.jpg" (Join-Path $RockDir "rock_diff.jpg")
Download-File "$RockBase/nor_gl_1k.jpg" (Join-Path $RockDir "rock_normal.jpg")
Download-File "$RockBase/rough_1k.jpg" (Join-Path $RockDir "rock_rough.jpg")

# Modelos glTF CC0 (Poly Haven) — pedras fotogramétricas
$BoulderGltf = Join-Path $ModelsDir "boulder_01.gltf"
if (-not (Test-Path $BoulderGltf)) {
    Write-Host "Modelos glTF: baixe de https://polyhaven.com/a/boulder_01 e extraia em assets/models/"
}

Write-Host "Assets prontos em $Root\assets"
