extern crate shellexpand;
use std::env;
use std::fs;
use std::io::Write;
use std::option_env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let payload_path = option_env!("PAYLOAD_PATH");
    println!("{:?}", payload_path);
    //Fix me: Need strip payload
    let payload_head = ".section .payload, \"ax\"
    .align 4
	.globl payload_bin
    payload_bin:
    ";

    let payload_content = if let Some(path) = payload_path {
        format!(".incbin	\"{}\"", shellexpand::full(path).unwrap())
    } else {
        " wfi
	    j	payload_bin"
            .to_string()
    };
    fs::File::create(out_dir.join("payload.S"))
        .unwrap()
        .write_fmt(format_args!("{} {}", payload_head, payload_content))
        .unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=htif.ld");
}
