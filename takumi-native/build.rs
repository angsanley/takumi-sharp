fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libtakumi")
        .csharp_class_accessibility("public")
        .csharp_namespace("TakumiSharp.Native")
        .csharp_class_name("Generated")
        .generate_csharp_file("../takumi-sharp/TakumiSharp.Native/TakumiSharp.g.cs")
        .unwrap()
}
