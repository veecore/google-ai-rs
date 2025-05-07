fn main() {
    println!("cargo::rustc-check-cfg=cfg(no_diagnostic_namespace)");
}
