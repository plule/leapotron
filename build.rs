use std::path::PathBuf;

use faust_build::build_dsp;

fn main() {
    dsp_regen();

    #[cfg(windows)]
    add_resources();

    gen_icon_data();
}

fn dsp_regen() {
    println!("cargo:rerun-if-changed=dsp");
    if std::env::var("THEREMOTION_REGEN_DSP").is_ok() {
        build_dsp("dsp/instrument.dsp");

        let mut generated = PathBuf::new();
        generated.push(std::env::var("OUT_DIR").unwrap());
        generated.push("dsp.rs");

        let mut dst = PathBuf::new();
        dst.push("src");
        dst.push("dsp.rs");

        std::fs::copy(generated, dst).unwrap();
    }
}

#[cfg(windows)]
fn add_resources() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().unwrap();
}

fn gen_icon_data() {
    println!("cargo:rerun-if-changed=assets/icon.png");
    const ICON: &[u8] = include_bytes!("assets/icon.png");
    let image = image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    let rgba = image.into_raw();
    let dst = format!("{}/icon", std::env::var("OUT_DIR").unwrap());
    std::fs::write(dst, rgba).unwrap();
}
