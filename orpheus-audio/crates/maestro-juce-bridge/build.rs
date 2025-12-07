use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .file("cpp/AudioEngine.cpp")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")  // MSVC
        .flag_if_supported("-O3")
        .flag_if_supported("/O2");        // MSVC

    // Add JUCE include paths when available
    // TODO: Uncomment when JUCE is integrated
    // let juce_dir = PathBuf::from("../../juce/modules");
    // if juce_dir.exists() {
    //     build.include(&juce_dir);
    // }

    build.compile("maestro_audio_engine");

    // Tell cargo to invalidate the built crate whenever the source changes
    println!("cargo:rerun-if-changed=cpp/AudioEngine.h");
    println!("cargo:rerun-if-changed=cpp/AudioEngine.cpp");

    // Link against C++ standard library
    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("windows") {
        // MSVC links automatically
    }
}
