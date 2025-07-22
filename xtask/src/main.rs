use std::{
    process::Command,
    // error::Error
};
use serde_json;

fn get_crate_name() -> String {
    match std::fs::read_to_string("Cargo.toml") {
        Ok(content) => {
            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                // 尝试获取 bin 名称
                if let Some(bin_array) = value.get("bin").and_then(|b| b.as_array()) {
                    if let Some(first_bin) = bin_array.first() {
                        if let Some(name) = first_bin.get("name").and_then(|n| n.as_str()) {
                            return name.to_string();
                        }
                    }
                }
                
                // 如果 bin 名称不存在，尝试获取 package 名称
                if let Some(package_name) = value.get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                {
                    return package_name.to_string();
                }
            }
            "unknown".to_string()
        }
        Err(_) => "unknown".to_string(),
    }
}

fn move_and_rename_binaries(targets: &[String], bin_name: &str) {
    //let pkg_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "".to_string());
    //let pkg_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "".to_string());
    //let pkg_env_n_abi = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "".to_string()) + &std::env::var("CARGO_CFG_TARGET_ABI").unwrap_or_else(|_| "".to_string());

    let dst_dir = "target/bin";
    if !std::path::Path::new(dst_dir).exists() {
        std::fs::create_dir_all(dst_dir).expect("Failed to create target/bin directory");
    }

    for target in targets {
        let bin_file = if target.contains("windows") {
            format!("{}.exe", bin_name)
        } else {
            bin_name.to_string()
        };
        let src_path = format!("target/{}/release/{}", target, bin_file);

        // 目标文件名包含 target 三元组
        let dst_file = format!(
            "{}-{}",
            bin_file, target
        );
        let dst_path = format!("{}/{}", dst_dir, dst_file);

        if std::path::Path::new(&src_path).exists() {
            std::fs::rename(&src_path, &dst_path)
                .expect(&format!("Failed to move and rename binary for target {}", target));
            println!("Moved and renamed binary to: {}", dst_path);

            // 如果是 linux 平台，压缩
            if target.contains("linux") {
                let tgz_path = format!("{}.tgz", dst_path);
                let tar_status = Command::new("tar")
                    .arg("czf")
                    .arg(&tgz_path)
                    .arg("-C")
                    .arg(dst_dir)
                    .arg(&dst_file)
                    .status()
                    .expect("Failed to execute tar for compression");

                if tar_status.success() {
                    std::fs::remove_file(&dst_path).expect("Failed to remove original binary after compression");
                    println!("Compressed to {} and removed original binary.", tgz_path);
                } else {
                    eprintln!("Compression failed for target {}", target);
                }
            }
        } else {
            eprintln!("Binary not found for target: {}", target);
        }
    }
}
fn main() {
    // 构建 targets 参数
    let targets: Vec<String> = match std::env::var("CARGO_TARGETS") {
        Ok(env_targets) if !env_targets.trim().is_empty() => {
            env_targets
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        }
        _ => {
            // 内置 target 列表（用 JSON 字符串数组）
            let builtin = r#"["x86_64-unknown-linux-musl","x86_64-unknown-linux-gnu"]"#;
            serde_json::from_str::<Vec<String>>(builtin).expect("Failed to parse builtin targets")
        }
    };

    let mut args: Vec<String> = vec!["build".to_string(), "--release".to_string()];
    for target in &targets {
        args.push("--target".to_string());
        args.push(target.clone());
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to execute cargo build");

    if !status.success() {
        eprintln!("cargo build failed");
        std::process::exit(1);
    }
    
    move_and_rename_binaries(&targets, &get_crate_name());
}