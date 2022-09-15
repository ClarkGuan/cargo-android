use std::env;
use std::process::Command;

pub fn run(target: &str) {
    let (api, args) = process_args();
    let _ = Command::new(env::var("CARGO").expect("env::var()"))
        .args(args)
        .envs(envs(target, api))
        .status()
        .expect("Command::status()");
}

fn process_args() -> (usize, Vec<String>) {
    let args = env::args().skip(2).collect::<Vec<_>>();
    let first = &args[0];
    if first.starts_with("api=") {
        ((&first[4..]).parse().unwrap(), (&args[1..]).to_vec())
    } else {
        (28, args)
    }
}

fn linker(s: &str) -> String {
    format!(
        "CARGO_TARGET_{}_LINKER",
        s.to_ascii_uppercase().replace('-', "_")
    )
}

fn runner(s: &str) -> String {
    format!(
        "CARGO_TARGET_{}_RUNNER",
        s.to_ascii_uppercase().replace('-', "_")
    )
}

fn ndk() -> String {
    const ENVS: &[&str] = &["NDK", "NDK_HOME", "NDK_ROOT"];
    ENVS.iter()
        .find_map(|item| env::var(*item).ok())
        .expect("no NDK envs")
}

#[allow(dead_code)]
struct Toolchain {
    cc: String,
    cxx: String,
    ar: String,
    strip: String,
}

fn toolchain(target: &str, api: usize) -> Toolchain {
    let arch = &target[..target.find("-").expect("target has no '-'")];
    let arch = if arch == "armv7" { "armv7a" } else { arch };
    let ndk = ndk();
    let host_os = target::os();
    let host_os = if host_os == "macos" {
        "darwin"
    } else {
        host_os
    };
    let host_arch = target::arch();
    Toolchain {
        cc: format!(
            "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android{}-clang",
            &ndk, host_os, host_arch, arch, api
        ),
        cxx: format!(
            "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android{}-clang++",
            &ndk, host_os, host_arch, arch, api
        ),
        ar: format!(
            "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android-ar",
            &ndk, host_os, host_arch, arch
        ),
        strip: format!(
            "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android-strip",
            &ndk, host_os, host_arch, arch
        ),
    }
}

fn envs(target: &str, api: usize) -> Vec<(String, String)> {
    let toolchain = toolchain(target, api);
    vec![
        ("CARGO_BUILD_TARGET".to_string(), target.to_string()),
        (linker(target), toolchain.cc.clone()),
        (runner(target), "arun".to_string()),
        (format!("CC_{}", target), toolchain.cc.clone()),
        (format!("CXX_{}", target), toolchain.cxx.clone()),
        (format!("AR_{}", target), toolchain.ar.clone()),
    ]
}
