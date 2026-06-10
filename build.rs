fn main() {
    // shaderc-sys compila glslang via CMake; versões novas do CMake exigem policy mínima.
    if std::env::var_os("CARGO_FEATURE_VULKAN").is_some() {
        println!("cargo:rustc-env=CMAKE_POLICY_VERSION_MINIMUM=3.5");
    }
}
