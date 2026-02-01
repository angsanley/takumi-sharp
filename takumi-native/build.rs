use std::{collections::HashMap, env, fs, path::PathBuf};
use syn::{Attribute, Fields, GenericArgument, Item, PathArguments, Type};

fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libtakumi")
        .csharp_namespace("TakumiSharp.Bindings")
        .csharp_class_name("NativeBindings")
        .generate_csharp_file("../takumi-sharp/TakumiSharp/Bindings/Bindings.g.cs")
        .unwrap();

    generate_csharp_models();
}

fn generate_csharp_models() {
    let takumi_path = find_takumi_source();

    let mut generator = CSharpGenerator::new();

    // Parse all .rs files in the node directory dynamically
    let node_dir = takumi_path.join("src/layout/node");
    if let Ok(entries) = fs::read_dir(&node_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                generator.parse_file(&path);
            }
        }
    }

    // Parse style properties from the define_style! macro
    let style_props = parse_style_properties(&takumi_path.join("src/layout/style/stylesheets.rs"));

    // Generate and write C# output
    let csharp_code = generator.generate(&style_props);
    fs::write(
        "../takumi-sharp/TakumiSharp/Models/Node.g.cs",
        csharp_code,
    )
    .expect("Failed to write Node.g.cs");

    println!("cargo:rerun-if-changed=build.rs");
}

/// Parse style properties from the define_style! macro in stylesheets.rs
fn parse_style_properties(path: &PathBuf) -> Vec<StyleProperty> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut properties = Vec::new();

    // Find the define_style! macro invocation
    if let Some(start) = content.find("define_style!(") {
        let macro_content = &content[start..];
        
        // Find the closing );
        let mut depth = 0;
        let mut end_pos = 0;
        for (i, c) in macro_content.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        let macro_body = &macro_content[14..end_pos]; // Skip "define_style!("
        
        // Parse each line for property definitions
        for line in macro_body.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }
            
            // Parse property: Type pattern
            // Format: property_name: Type,
            // or: property_name: Type where inherit = true,
            if let Some(colon_pos) = line.find(':') {
                let prop_name = line[..colon_pos].trim();
                
                // Skip attribute lines
                if prop_name.starts_with('#') || prop_name.is_empty() {
                    continue;
                }
                
                let rest = &line[colon_pos + 1..];
                
                // Find the type (ends at comma or "where")
                let type_end = rest.find(" where")
                    .or_else(|| rest.rfind(','))
                    .unwrap_or(rest.len());
                
                let rust_type = rest[..type_end].trim().trim_end_matches(',');
                
                if !prop_name.is_empty() && !rust_type.is_empty() {
                    properties.push(StyleProperty {
                        name: prop_name.to_string(),
                        rust_type: rust_type.to_string(),
                        csharp_type: style_rust_type_to_csharp(rust_type),
                    });
                }
            }
        }
    }

    properties
}

/// Dynamically maps Rust types to C# types.
/// Uses pattern matching on type structure, not hardcoded type names.
fn style_rust_type_to_csharp(rust_ty: &str) -> String {
    // Handle Option<T> - make nullable
    if let Some(inner) = rust_ty.strip_prefix("Option<").and_then(|s| s.strip_suffix(">")) {
        return format!("{}?", style_rust_type_to_csharp(inner).trim_end_matches('?'));
    }
    
    // Handle Vec<T> -> List<T>
    if let Some(inner) = rust_ty.strip_prefix("Vec<").and_then(|s| s.strip_suffix(">")) {
        return format!("List<{}>?", style_rust_type_to_csharp(inner).trim_end_matches('?'));
    }
    
    // Handle Box<[T]> -> List<T>
    if let Some(inner) = rust_ty.strip_prefix("Box<[").and_then(|s| s.strip_suffix("]>")) {
        return format!("List<{}>?", style_rust_type_to_csharp(inner).trim_end_matches('?'));
    }

    // Primitive types - these are universal Rust types, not takumi-specific
    match rust_ty {
        "f32" => "float?".to_string(),
        "f64" => "double?".to_string(),
        "i32" => "int?".to_string(),
        "i64" => "long?".to_string(),
        "u32" => "uint?".to_string(),
        "u64" => "ulong?".to_string(),
        "bool" => "bool?".to_string(),
        "String" | "str" => "string?".to_string(),
        // Everything else is treated as a complex type -> object?
        // This is the safest fallback for CSS values that can have multiple formats
        _ => "object?".to_string(),
    }
}

#[derive(Debug, Clone)]
struct StyleProperty {
    name: String,
    #[allow(dead_code)]
    rust_type: String,
    csharp_type: String,
}

fn find_takumi_source() -> PathBuf {
    // Find takumi in cargo registry
    let home = env::var("HOME").expect("HOME not set");
    let cargo_registry = PathBuf::from(&home).join(".cargo/registry/src");

    // Find the latest takumi version
    for entry in fs::read_dir(&cargo_registry).expect("Cannot read cargo registry") {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.is_dir() {
            for pkg in fs::read_dir(&path).expect("Cannot read index dir") {
                let pkg = pkg.expect("Cannot read pkg");
                let pkg_path = pkg.path();
                if pkg_path.is_dir() {
                    let name = pkg_path.file_name().unwrap().to_string_lossy();
                    if name.starts_with("takumi-0.") {
                        return pkg_path;
                    }
                }
            }
        }
    }
    panic!("Could not find takumi source in cargo registry");
}

#[derive(Debug, Clone)]
struct StructDef {
    name: String,
    fields: Vec<FieldDef>,
    #[allow(dead_code)]
    serde_attrs: SerdeAttrs,
}

#[derive(Debug, Clone)]
struct FieldDef {
    name: String,
    ty: String,
    csharp_ty: String,
}

#[derive(Debug, Clone)]
struct EnumDef {
    name: String,
    variants: Vec<EnumVariant>,
    #[allow(dead_code)]
    serde_attrs: SerdeAttrs,
}

#[derive(Debug, Clone)]
struct EnumVariant {
    name: String,
    inner_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct SerdeAttrs {
    tag: Option<String>,
    rename_all: Option<String>,
}

struct CSharpGenerator {
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
}

impl CSharpGenerator {
    fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    fn parse_file(&mut self, path: &PathBuf) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not read {:?}: {}", path, e);
                return;
            }
        };

        let file = match syn::parse_file(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: Could not parse {:?}: {}", path, e);
                return;
            }
        };

        for item in file.items {
            match item {
                Item::Struct(s) => self.process_struct(s),
                Item::Enum(e) => self.process_enum(e),
                _ => {}
            }
        }
    }

    fn process_struct(&mut self, s: syn::ItemStruct) {
        let name = s.ident.to_string();

        // Process structs that end with "Node" - this is a pattern, not a hardcoded list
        if !name.ends_with("Node") {
            return;
        }

        let serde_attrs = parse_serde_attrs(&s.attrs);
        let mut fields = Vec::new();

        if let Fields::Named(named) = s.fields {
            for field in named.named {
                let field_name = field.ident.map(|i| i.to_string()).unwrap_or_default();
                let rust_ty = type_to_string(&field.ty);
                let csharp_ty = rust_type_to_csharp(&rust_ty);

                fields.push(FieldDef {
                    name: field_name,
                    ty: rust_ty,
                    csharp_ty,
                });
            }
        }

        self.structs.insert(
            name.clone(),
            StructDef {
                name,
                fields,
                serde_attrs,
            },
        );
    }

    fn process_enum(&mut self, e: syn::ItemEnum) {
        let name = e.ident.to_string();

        // Process enums that end with "Kind" - this is a pattern for tagged unions
        if !name.ends_with("Kind") {
            return;
        }

        let serde_attrs = parse_serde_attrs(&e.attrs);
        let mut variants = Vec::new();

        for variant in e.variants {
            let variant_name = variant.ident.to_string();
            let inner_type = match variant.fields {
                Fields::Unnamed(fields) => {
                    fields.unnamed.first().map(|f| type_to_string(&f.ty))
                }
                _ => None,
            };

            variants.push(EnumVariant {
                name: variant_name,
                inner_type,
            });
        }

        self.enums.insert(
            name.clone(),
            EnumDef {
                name,
                variants,
                serde_attrs,
            },
        );
    }

    fn generate(&self, style_props: &[StyleProperty]) -> String {
        let mut output = String::new();

        // Header
        output.push_str("// <auto-generated>\n");
        output.push_str("// This file was generated by takumi-native build.rs\n");
        output.push_str("// Do not edit manually.\n");
        output.push_str("// </auto-generated>\n\n");
        output.push_str("#nullable enable\n\n");
        output.push_str("using System.Text.Json;\n");
        output.push_str("using System.Text.Json.Serialization;\n\n");
        output.push_str("namespace TakumiSharp.Models;\n\n");

        // Generate all "Kind" enums as polymorphic base classes
        for (enum_name, enum_def) in &self.enums {
            // Generate base class name by removing "Kind" suffix
            let base_class_name = enum_name.trim_end_matches("Kind");
            if base_class_name.is_empty() {
                continue;
            }

            output.push_str("[JsonPolymorphic(TypeDiscriminatorPropertyName = \"type\")]\n");

            for variant in &enum_def.variants {
                // Derive the C# type name from the variant's inner type
                let csharp_type = if let Some(inner) = &variant.inner_type {
                    // Extract the base type name (e.g., "ContainerNode<NodeKind>" -> "ContainerNode")
                    inner.split('<').next().unwrap_or(inner).to_string()
                } else {
                    format!("{}Node", variant.name)
                };
                
                // Discriminator value is lowercase variant name
                let tag = variant.name.to_lowercase();
                output.push_str(&format!(
                    "[JsonDerivedType(typeof({}), \"{}\")]\n",
                    csharp_type, tag
                ));
            }

            output.push_str(&format!("public abstract class {} {{ }}\n\n", enum_name));
        }

        // Generate all node structs dynamically
        let mut node_names: Vec<_> = self.structs.keys().collect();
        node_names.sort(); // Deterministic output order
        
        for name in node_names {
            if let Some(struct_def) = self.structs.get(name) {
                output.push_str(&self.generate_class(struct_def));
                output.push('\n');
            }
        }

        // Generate Style class with all properties
        output.push_str(&Self::generate_style_class(style_props));
        output.push('\n');

        // Generate TailwindValues as a simple string alias
        output.push_str("/// <summary>\n");
        output.push_str("/// Tailwind CSS classes string. Parsed and applied at runtime.\n");
        output.push_str("/// </summary>\n");
        output.push_str("[JsonConverter(typeof(TailwindValuesConverter))]\n");
        output.push_str("public class TailwindValues\n{\n");
        output.push_str("    public string Value { get; set; } = string.Empty;\n\n");
        output.push_str("    public TailwindValues() { }\n");
        output.push_str("    public TailwindValues(string value) => Value = value;\n\n");
        output.push_str("    public static implicit operator TailwindValues(string value) => new(value);\n");
        output.push_str("    public static implicit operator string(TailwindValues tw) => tw.Value;\n");
        output.push_str("}\n\n");

        // Generate TailwindValuesConverter
        output.push_str("public class TailwindValuesConverter : JsonConverter<TailwindValues>\n{\n");
        output.push_str("    public override TailwindValues? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)\n");
        output.push_str("        => reader.GetString() is string s ? new TailwindValues(s) : null;\n\n");
        output.push_str("    public override void Write(Utf8JsonWriter writer, TailwindValues value, JsonSerializerOptions options)\n");
        output.push_str("        => writer.WriteStringValue(value.Value);\n");
        output.push_str("}\n");

        output
    }

    fn generate_style_class(style_props: &[StyleProperty]) -> String {
        let mut output = String::new();

        output.push_str("/// <summary>\n");
        output.push_str("/// CSS-like style properties for layout and rendering.\n");
        output.push_str("/// </summary>\n");
        output.push_str("public class Style\n{\n");

        for prop in style_props {
            let json_name = to_camel_case(&prop.name);
            let prop_name = to_pascal_case(&prop.name);
            
            output.push_str(&format!(
                "    [JsonPropertyName(\"{}\")]\n",
                json_name
            ));
            output.push_str("    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]\n");
            output.push_str(&format!(
                "    public {} {} {{ get; set; }}\n\n",
                prop.csharp_type, prop_name
            ));
        }

        output.push_str("}\n");
        output
    }

    fn generate_class(&self, struct_def: &StructDef) -> String {
        let mut output = String::new();

        // Find the base class - look for a "Kind" enum that references this struct
        let base_class = self.find_base_class_for_struct(&struct_def.name);

        if let Some(base) = base_class {
            output.push_str(&format!(
                "public class {} : {}\n{{\n",
                struct_def.name, base
            ));
        } else {
            output.push_str(&format!(
                "public class {}\n{{\n",
                struct_def.name
            ));
        }

        for field in &struct_def.fields {
            let json_name = to_camel_case(&field.name);
            output.push_str(&format!(
                "    [JsonPropertyName(\"{}\")]\n",
                json_name
            ));
            output.push_str("    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]\n");
            
            let prop_name = to_pascal_case(&field.name);
            
            // Determine the C# type, handling special recursive cases
            let csharp_ty = self.resolve_csharp_type(&field.ty, &field.csharp_ty);
            
            // Add default value for non-nullable strings
            let default_value = if csharp_ty == "string" {
                " = string.Empty;"
            } else {
                ""
            };
            output.push_str(&format!(
                "    public {} {} {{ get; set; }}{}\n\n",
                csharp_ty, prop_name, default_value
            ));
        }

        output.push_str("}\n");
        output
    }

    /// Find the base class for a struct by checking if any "Kind" enum references it
    fn find_base_class_for_struct(&self, struct_name: &str) -> Option<String> {
        for (enum_name, enum_def) in &self.enums {
            for variant in &enum_def.variants {
                if let Some(inner) = &variant.inner_type {
                    // Check if the inner type matches this struct
                    let base_type = inner.split('<').next().unwrap_or(inner);
                    if base_type == struct_name {
                        return Some(enum_name.clone());
                    }
                }
            }
        }
        None
    }

    /// Resolve C# type, handling recursive node types
    fn resolve_csharp_type(&self, rust_ty: &str, default_csharp: &str) -> String {
        // Check if this is a recursive node type (contains generic Node parameter)
        if rust_ty.contains("Nodes") || rust_ty.contains("Node<") {
            // This is a children field - find the appropriate Kind enum
            if let Some(enum_name) = self.enums.keys().next() {
                return format!("List<{}>?", enum_name);
            }
        }
        
        default_csharp.to_string()
    }
}

fn parse_serde_attrs(attrs: &[Attribute]) -> SerdeAttrs {
    let mut result = SerdeAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("tag") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.tag = Some(value.value());
                } else if meta.path.is_ident("rename_all") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename_all = Some(value.value());
                }
                Ok(())
            });
        }
    }

    result
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<String> = type_path
                .path
                .segments
                .iter()
                .map(|seg| {
                    let ident = seg.ident.to_string();
                    match &seg.arguments {
                        PathArguments::AngleBracketed(args) => {
                            let inner: Vec<String> = args
                                .args
                                .iter()
                                .filter_map(|arg| match arg {
                                    GenericArgument::Type(t) => Some(type_to_string(t)),
                                    _ => None,
                                })
                                .collect();
                            if inner.is_empty() {
                                ident
                            } else {
                                format!("{}<{}>", ident, inner.join(", "))
                            }
                        }
                        _ => ident,
                    }
                })
                .collect();
            segments.join("::")
        }
        // Handle slice types like [Nodes] -> represent as [Nodes]
        Type::Slice(slice) => {
            let inner = type_to_string(&slice.elem);
            format!("[{}]", inner)
        }
        _ => "unknown".to_string(),
    }
}

fn rust_type_to_csharp(rust_ty: &str) -> String {
    // Handle Option<T>
    if let Some(inner) = rust_ty.strip_prefix("Option<").and_then(|s| s.strip_suffix(">")) {
        let inner_csharp = rust_type_to_csharp(inner);
        return format!("{}?", inner_csharp.trim_end_matches('?'));
    }

    // Handle Box<[T]> -> List<T>
    if let Some(inner) = rust_ty.strip_prefix("Box<[").and_then(|s| s.strip_suffix("]>")) {
        let inner_csharp = rust_type_to_csharp(inner);
        return format!("List<{}>", inner_csharp.trim_end_matches('?'));
    }

    // Handle Vec<T> -> List<T>
    if let Some(inner) = rust_ty.strip_prefix("Vec<").and_then(|s| s.strip_suffix(">")) {
        let inner_csharp = rust_type_to_csharp(inner);
        return format!("List<{}>", inner_csharp.trim_end_matches('?'));
    }

    // Handle Arc<str> -> string
    if rust_ty == "Arc<str>" {
        return "string".to_string();
    }

    // Primitive types - these are standard Rust types, not library-specific
    match rust_ty {
        "String" => "string".to_string(),
        "str" => "string".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        "i32" => "int".to_string(),
        "i64" => "long".to_string(),
        "u32" => "uint".to_string(),
        "u64" => "ulong".to_string(),
        "bool" => "bool".to_string(),
        // Known model types we generate
        "Style" => "Style".to_string(),
        "TailwindValues" => "TailwindValues".to_string(),
        // Everything else - use the Rust type name as-is (will be object for unknown)
        _ => rust_ty.to_string(),
    }
}

fn to_camel_case(s: &str) -> String {
    // snake_case to camelCase
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn to_pascal_case(s: &str) -> String {
    let camel = to_camel_case(s);
    let mut chars = camel.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
