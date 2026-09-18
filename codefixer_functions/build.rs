fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/run_limited.c");
    println!("cargo::rustc-link-lib=seccomp");

    cc::Build::new()
        .file("src/run_limited.c")
        .flag("-O2")
        .compile("run_limited");
}
