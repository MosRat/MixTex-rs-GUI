use std::env;
use std::path::PathBuf;

fn main() {
    let lib_path = env::var("GEX_LIB_PATH").expect("GEX_LIB_PATH environment variable not set");
    let lib_path = PathBuf::from(lib_path);

    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // 根据平台选择链接的库名
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=libgex");
    } else {
        println!("cargo:rustc-link-lib=dylib=gex");
    }

    // 告诉 Cargo 如果环境变量变化需要重新构建
    println!("cargo:rerun-if-env-changed=GEX_LIB_PATH");

    // 构建完成后拷贝库文件到输出目录
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.parent().unwrap().parent().unwrap().parent().unwrap();

    let lib_name = if cfg!(target_os = "windows") {
        "libgex.dll"
    } else if cfg!(target_os = "macos") {
        "libgex.dylib"
    } else {
        "libgex.so"
    };

    let src = lib_path.join(lib_name);
    let dst = target_dir.join(lib_name);
    let dst_bin_dir = target_dir.parent().unwrap().parent().unwrap().join("bin");
    std::fs::create_dir_all(&dst_bin_dir).unwrap();
    let dst_bin = dst_bin_dir.join(lib_name);

    if src.exists() {
        std::fs::copy(&src, &dst).unwrap_or_else(|_| {
            panic!("Failed to copy {} to {}", src.display(), dst.display())
        });        
        std::fs::copy(&src, &dst_bin).unwrap_or_else(|_| {
            panic!("Failed to copy {} to {}", src.display(), dst.display())
        });
    }
    // #[cfg(target_os = "windows")]
    // {     
    //     let lib_name = "openblas.dll";
    //     let src = lib_path.join(lib_name);
    //     let dst_bin = dst_bin_dir.join(lib_name);
    //     std::fs::copy(&src, &dst_bin).unwrap_or_else(|_| {
    //         panic!("Failed to copy {} to {}", src.display(), dst.display())
    //     });
    // }
}