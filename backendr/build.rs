// 编译期注入 rustc 版本供 server-monitor 上报（对齐 Go runtime.Version()）。
// 用法：option_env!("GOWIND_RUSTC_VERSION")。

fn main() {
    println!("cargo:rustc-env=GOWIND_RUSTC_VERSION={}", rustc_version());
    println!("cargo:rerun-if-changed=build.rs");
}

fn rustc_version() -> String {
    let mut cmd = std::process::Command::new("rustc");
    cmd.arg("--version");
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let v = s.trim();
            if !v.is_empty() {
                return v.to_string();
            }
        }
    }
    format!("rust {}", env!("CARGO_PKG_VERSION"))
}