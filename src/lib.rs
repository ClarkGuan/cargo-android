use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn run(target: &str) {
    let (api, args) = process_args();
    let api = if api < 19 { 19 } else { api };
    let _ = Command::new(env::var("CARGO").expect("no CARGO env var"))
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
        .expect("no $NDK vars")
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
    let ndk = ndk();
    let host_os = target::os();
    let host_os = if host_os == "macos" {
        "darwin"
    } else {
        host_os
    };
    let host_arch = target::arch();
    match arch {
        "armv7" => Toolchain {
            cc: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/armv7a-linux-androideabi{}-clang",
                &ndk, host_os, host_arch, api
            ),
            cxx: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/armv7a-linux-androideabi{}-clang++",
                &ndk, host_os, host_arch, api
            ),
            ar: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/llvm-ar",
                &ndk, host_os, host_arch
            ),
            strip: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/llvm-strip",
                &ndk, host_os, host_arch
            ),
        },

        arch => Toolchain {
            cc: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android{}-clang",
                &ndk, host_os, host_arch, arch, api
            ),
            cxx: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/{}-linux-android{}-clang++",
                &ndk, host_os, host_arch, arch, api
            ),
            ar: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/llvm-ar",
                &ndk, host_os, host_arch
            ),
            strip: format!(
                "{}/toolchains/llvm/prebuilt/{}-{}/bin/llvm-strip",
                &ndk, host_os, host_arch
            ),
        },
    }
}

fn envs(target: &str, api: usize) -> Vec<(String, String)> {
    let toolchain = toolchain(target, api);
    vec![
        ("CARGO_BUILD_TARGET".to_string(), target.to_string()),
        (linker(target), toolchain.cc.clone()),
        (runner(target), "cargo-ndk-run".to_string()),
        (format!("CC_{}", target), toolchain.cc.clone()),
        (format!("CXX_{}", target), toolchain.cxx.clone()),
        (format!("AR_{}", target), toolchain.ar.clone()),
    ]
}

pub fn adb() -> bool {
    which::which("adb").is_ok()
}

pub fn is_file<P: AsRef<Path>>(p: P) -> bool {
    p.as_ref().is_file()
}

pub fn adb_push<S: AsRef<OsStr>>(s: S) -> bool {
    matches!(Command::new("adb")
        .arg("push")
        .arg(s)
        .arg("/data/local/tmp/")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status(), Ok(s) if s.success())
}

pub fn adb_remove_all() -> bool {
    matches!(Command::new("adb").arg("shell").arg("sh -c 'rm -rf /data/local/tmp/*'").status(),
        Ok(s) if s.success())
}

pub fn adb_run<S, I>(target: S, args: I) -> bool
where
    S: AsRef<OsStr>,
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    let prefix_args = vec![
        "shell".as_ref(),
        "cd /data/local/tmp/".as_ref(),
        "&& chmod 777".as_ref(),
        target.as_ref(),
        "&& time sh -c 'LD_LIBRARY_PATH=/data/local/tmp RUST_BACKTRACE=full".as_ref(),
        target.as_ref(),
    ];
    let mut cmd = Command::new("adb");
    cmd.args(prefix_args)
        .args(args)
        .arg("; echo \"============================\n[exit status:($?)]\" 1>&2'");
    matches!(cmd.status(), Ok(s) if s.success())
}

pub fn path_of_device<P: AsRef<Path>>(p: P) -> PathBuf {
    let name = p.as_ref().file_name().expect("no filename");
    "/data/local/tmp"
        .parse::<PathBuf>()
        .expect("&str -> PathBuf")
        .join(name)
}
