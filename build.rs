use wgsl_grease::{Error, wgpu_types};

fn main() -> Result<(), Box<Error>> {
    println!("cargo::rerun-if-changed=src/render/shaders/draw.wgsl");
    println!("cargo::rerun-if-changed=src/render/shaders/common.wgsl");

    if !std::fs::exists("src/render/shaders/out").unwrap() {
        std::fs::create_dir("src/render/shaders/out").unwrap();
    }

    wgsl_grease::WgslBindgenBuilder::default()
        .shader_root("src/render/shaders")
        .add_shader("common.wgsl")
        .add_shader("draw.wgsl")
        .output("src/render/shaders/out")
        .separate_files(true)
        .build()
        .inspect_err(|e| eprintln!("{e:#?}"))?
        .generate()?;

    Ok(())
}
