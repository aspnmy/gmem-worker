# 构建项目
Write-Host "Building project..."
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# 读取大版本号
$versionFile = "ver"
Write-Host "Checking version file: $versionFile"
if (Test-Path $versionFile) {
    $versionStr = (Get-Content $versionFile -Raw).Trim()
    Write-Host "Version from file: $versionStr"
} else {
    $versionStr = "0.1.0"
    Write-Host "Version file not found, using default: $versionStr"
}

# 生成时间戳（YYYYMMDDHHSS）
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
if ([string]::IsNullOrEmpty($timestamp)) {
    # 备用方法
    $now = [DateTime]::Now
    $timestamp = $now.ToString("yyyyMMddHHmmss")
}
Write-Host "Timestamp: $timestamp"

# 构建完整版本号
Write-Host "versionStr type: $($versionStr.GetType().Name)"
Write-Host "versionStr value: '$versionStr'"
Write-Host "timestamp type: $($timestamp.GetType().Name)"
Write-Host "timestamp value: '$timestamp'"
$fullVersion = "v" + $versionStr + "-" + $timestamp
Write-Host "Full version: $fullVersion"

# 定义文件路径
$sourcePath = ".\target\release\GmemoryStore.exe"
$destPath = ".\target\release\GmemoryStore_${fullVersion}.exe"
Write-Host "Source path: $sourcePath"
Write-Host "Destination path: $destPath"

# 检查源文件是否存在
if (-not (Test-Path $sourcePath)) {
    Write-Host "Source executable not found!" -ForegroundColor Red
    exit 1
}

# 重命名文件
Write-Host "Renaming executable to: $destPath"
Copy-Item $sourcePath $destPath -Force

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build and rename completed successfully!" -ForegroundColor Green
    Write-Host "Executable: $destPath"
} else {
    Write-Host "Rename failed!" -ForegroundColor Red
    exit 1
}
