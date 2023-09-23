use faust_build::build_dsp;
use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "leap")]
    setup_leapsdk_link();
    dsp_regen();

    #[cfg(windows)]
    add_resources();

    gen_icon_data();
}

#[cfg(feature = "leap")]
fn setup_leapsdk_link() {
    #[cfg(windows)]
    const DEFAULT_LEAPSDK_LIB_PATH: &str = r"C:\Program Files\Ultraleap\LeapSDK\lib\x64";

    #[cfg(not(windows))]
    const DEFAULT_LEAPSDK_LIB_PATH: &str = r"/usr/share/doc/ultraleap-hand-tracking-service";
    // Find Leap SDK
    println!(r"cargo:rerun-if-env-changed=LEAPSDK_LIB_PATH");

    let leapsdk_path =
        env::var("LEAPSDK_LIB_PATH").unwrap_or_else(|_| DEFAULT_LEAPSDK_LIB_PATH.to_string());

    let leapsdk_path = PathBuf::from(leapsdk_path);

    if !leapsdk_path.is_dir() {
        println!("cargo:warning=Could not find LeapSDK at the location {}. Install it from https://developer.leapmotion.com/tracking-software-download or set its location with the environment variable LEAPSDK_LIB_PATH.", leapsdk_path.display());
    } else {
        let path_str = leapsdk_path
            .to_str()
            .unwrap_or_else(|| panic!("{} is not a valid path.", leapsdk_path.display()));

        // Link to LeapC.lib
        println!(r"cargo:rustc-link-lib=LeapC");
        println!(r"cargo:rustc-link-search={}", path_str);
    }
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
