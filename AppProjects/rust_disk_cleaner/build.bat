@echo off
cd "%~dp0"
echo 编译C盘清理工具...
cargo build
if %errorlevel% equ 0 (
    echo 编译成功！
    echo 可执行文件位置：target\debug\disk_cleaner.exe
) else (
    echo 编译失败，请检查错误信息
)
pause