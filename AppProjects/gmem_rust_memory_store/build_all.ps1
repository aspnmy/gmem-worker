# 一次性编译所有版本的脚本
# 编译库和所有二进制目标

Write-Host "开始编译 gmem_rust_memory_store 项目的所有版本..." -ForegroundColor Green

# 1. 编译库（默认特性）
Write-Host "\n1. 编译库（默认特性）..." -ForegroundColor Cyan
try {
    cargo build
    Write-Host "✓ 库编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 库编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 2. 编译库（完整特性）
Write-Host "\n2. 编译库（完整特性）..." -ForegroundColor Cyan
try {
    cargo build --features full
    Write-Host "✓ 库（完整特性）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 库（完整特性）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 3. 编译所有二进制目标（开发版本）
Write-Host "\n3. 编译所有二进制目标（开发版本）..." -ForegroundColor Cyan
try {
    cargo build --bins
    Write-Host "✓ 二进制目标（开发版本）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 二进制目标（开发版本）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 4. 编译所有二进制目标（完整特性，开发版本）
Write-Host "\n4. 编译所有二进制目标（完整特性，开发版本）..." -ForegroundColor Cyan
try {
    cargo build --bins --features full
    Write-Host "✓ 二进制目标（完整特性，开发版本）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 二进制目标（完整特性，开发版本）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 5. 编译库（发行版本，默认特性）
Write-Host "\n5. 编译库（发行版本，默认特性）..." -ForegroundColor Cyan
try {
    cargo build --release
    Write-Host "✓ 库（发行版本）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 库（发行版本）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 6. 编译库（发行版本，完整特性）
Write-Host "\n6. 编译库（发行版本，完整特性）..." -ForegroundColor Cyan
try {
    cargo build --release --features full
    Write-Host "✓ 库（发行版本，完整特性）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 库（发行版本，完整特性）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 7. 编译所有二进制目标（发行版本）
Write-Host "\n7. 编译所有二进制目标（发行版本）..." -ForegroundColor Cyan
try {
    cargo build --bins --release
    Write-Host "✓ 二进制目标（发行版本）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 二进制目标（发行版本）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 8. 编译所有二进制目标（完整特性，发行版本）
Write-Host "\n8. 编译所有二进制目标（完整特性，发行版本）..." -ForegroundColor Cyan
try {
    cargo build --bins --release --features full
    Write-Host "✓ 二进制目标（完整特性，发行版本）编译成功" -ForegroundColor Green
} catch {
    Write-Host "✗ 二进制目标（完整特性，发行版本）编译失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 9. 运行测试
Write-Host "\n9. 运行测试..." -ForegroundColor Cyan
try {
    cargo test
    Write-Host "✓ 测试通过" -ForegroundColor Green
} catch {
    Write-Host "✗ 测试失败: $($_.Exception.Message)" -ForegroundColor Red
    # 测试失败不阻止构建完成
}

Write-Host "\n🎉 所有版本编译完成！" -ForegroundColor Green
Write-Host "\n编译结果位置：" -ForegroundColor Yellow
Write-Host "- 开发版本：target/debug/" -ForegroundColor White
Write-Host "- 发行版本：target/release/" -ForegroundColor White
