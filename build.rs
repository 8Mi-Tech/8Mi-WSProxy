use std::env;
use std::fs;
use std::path::Path;
use cargo_metadata::{MetadataCommand, Package};

fn main() {
    // 获取 OUT_DIR 环境变量
    let out_dir = env::var("OUT_DIR").unwrap();

    // 指定你想要的输出目录
    let target_binary_dir = Path::new("target/binarys");
    fs::create_dir_all(&target_binary_dir).unwrap();

    // 获取 Cargo.toml 的元数据
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    // 获取当前包
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let package: &Package = metadata
        .packages
        .iter()
        .find(|p| p.name.as_str() == package_name)
        .expect("Package not found");

    // 获取目标三元组信息
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "unknown".to_string());
    let target_abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_else(|_| "".to_string());

    // 遍历 bin 目标
    for bin in &package.targets {
        if bin.kind.iter().any(|k| k.to_string() == "bin") {
            let binary_name = &bin.name;
            let src_path = Path::new(&out_dir).join(binary_name);
            // 判断是否为 gnullvm
            let is_gnullvm = target_env == "gnu" && target_abi == "llvm";

            // 生成目标文件名
            // 如果是 macos 或 gnullvm，则不带 target_env 标签
            let new_file_name = if target_os == "macos" || is_gnullvm {
                format!("{}-{}-{}", binary_name, target_os, target_arch)
            } else {
                format!("{}-{}-{}-{}", binary_name, target_os, target_arch, target_env)
            };
            let dst_path = target_binary_dir.join(new_file_name);

            if src_path.exists() {
                fs::copy(&src_path, &dst_path).unwrap();
                println!("cargo:warning=Binary copied to {:?}", dst_path);
            }
        }
    }
}
