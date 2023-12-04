fn main() {
    cc::Build::new()
        .file("src/bindings.cpp")
        .include("../../subprojects/llama.cpp")
        .include("../../subprojects/llama.cpp/common")
        .include("../../subprojects/llama.cpp/examples/llava")
        .compile("bindings");
}
