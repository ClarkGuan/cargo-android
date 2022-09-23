use scopeguard::defer;
use std::env;

fn main() {
    let args = env::args().skip(2).collect::<Vec<_>>();
    assert!(args.len() > 0, "args.len() == 0");
    assert!(cargo_android::adb(), "adb not exist in $PATH");
    let (files, args) = partition(&args);
    assert!(
        files.len() > 0 && cargo_android::is_file(&files[0]),
        "runnable file not found"
    );
    defer! {
        assert!(cargo_android::adb_remove_all(), "adb_remove_all failed!");
    }
    for f in files {
        assert!(cargo_android::adb_push(f), "adb push error: {}", f);
    }
    assert!(
        cargo_android::adb_run(cargo_android::path_of_device(&files[0]), args),
        "adb run error!"
    );
}

fn partition(args: &[String]) -> (&[String], &[String]) {
    match args
        .iter()
        .enumerate()
        .find_map(|(i, s)| if s == "--" { Some(i) } else { None })
    {
        Some(index) => (&args[..index], &args[index + 1..]),
        _ => (args, &[]),
    }
}
