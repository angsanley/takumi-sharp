fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libtakumi")
        .csharp_namespace("TakumiSharp.Bindings")
        .csharp_class_name("NativeBindings")
        .generate_csharp_file("../takumi-sharp/TakumiSharp/Bindings/Bindings.g.cs")
        .unwrap()
}
