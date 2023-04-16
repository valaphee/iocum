use std::path::PathBuf;

fn main() {
    prost_build::Config::new()
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"),
        )
        .compile_protos(
            &glob::glob("src/**/*.proto")
                .unwrap()
                .map(|path| path.unwrap())
                .collect::<Vec<_>>(),
            &["src/"],
        )
        .unwrap();
}
