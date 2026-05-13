fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");

    let sha = std::env::var("GIT_SHA").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=GIT_SHA={sha}");
}
