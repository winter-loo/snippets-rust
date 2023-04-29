use std::env;
use std::io::Result;
use std::process::Command;

fn main() -> Result<()> {
    if cfg!(use_prebuilt_protoc_binary) {
        // use -vv to see its output
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("out dir is {out_dir}");
        let protoc_binary_dir = "protoc";
        let protoc_path = format!("{out_dir}/{protoc_binary_dir}/bin/protoc");
        println!("protoc path: {protoc_path}");
        env::set_var("PROTOC", protoc_path);

        let output = Command::new("sh")
            .arg("download_protoc.sh")
            .arg(protoc_binary_dir)
            .output()
            .expect("Failed to execute script");

        println!(
            "shell command output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        // protobuf-src must be in `build-dependencies` section of Cargo.toml
        env::set_var("PROTOC", protobuf_src::protoc());
        prost_build::compile_protos(&["src/stats.proto"], &["src/"])?;
    }
    Ok(())
}
