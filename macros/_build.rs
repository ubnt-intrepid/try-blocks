use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitStatus},
};

fn main() {
    if probe_try_blocks().map_or(false, |st| st.success()) {
        println!("cargo:rustc-cfg=try_blocks");
    }
}

fn probe_try_blocks() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR").map(PathBuf::from)?;
    let probefile = out_dir.join("probe_try_blocks.rs");

    fs::write(
        &probefile,
        r#"
            #![feature(try_blocks)]

            fn foo() -> Result<(), ()> {
                Ok(())
            }

            fn main() {
                let _: Result<(), ()> = try {
                    foo()?;
                };
            }
        "#,
    )
    .ok()?;

    Command::new(&rustc)
        .arg("--edition=2018")
        .arg("--crate-name=build_probe_try_blocks")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}
