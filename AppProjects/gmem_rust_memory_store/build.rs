use std::fs;

fn main() {
    // 读取大版本号
    let version_file = "ver";
    let version_str = match fs::read_to_string(version_file) {
        Ok(content) => content.trim().to_string(),
        Err(_) => "0.1.0".to_string(),
    };

    // 生成时间戳（YYYYMMDDHHSS）
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();

    // 构建完整版本号
    let full_version = format!("{}-{}", version_str, timestamp);

    // 设置环境变量，供代码使用
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", full_version);
    println!("cargo:rustc-env=APP_VERSION={}", full_version);
    println!("cargo:rustc-env=APP_VERSION_MAJOR={}", version_str);
    println!("cargo:rustc-env=APP_VERSION_TIMESTAMP={}", timestamp);

    // 处理Windows平台的图标和版本信息
    if cfg!(target_os = "windows") {
        embed_resource::compile("icon.rc");
    }

    // 配置图标
    println!("cargo:rerun-if-changed=E://个人编程习惯记忆库//exe_ico//devrom.ico");
}