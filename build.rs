use std::{env, path::PathBuf};

use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
struct EnumRenameCallbacks;

impl ParseCallbacks for EnumRenameCallbacks {
    fn will_parse_macro(&self, _name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        bindgen::callbacks::MacroParsingBehavior::Default
    }

    fn generated_name_override(
        &self,
        _item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        None
    }

    fn generated_link_name_override(
        &self,
        _item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        None
    }

    fn int_macro(&self, _name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        None
    }

    fn str_macro(&self, _name: &str, _value: &[u8]) {}

    fn func_macro(&self, _name: &str, _value: &[&[u8]]) {}

    fn enum_variant_behavior(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<bindgen::callbacks::EnumVariantCustomBehavior> {
        None
    }

    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let mut name = original_variant_name.to_string();
        if name.starts_with("AHARDWAREBUFFER_") {
            name = name.replace("AHARDWAREBUFFER_", "")
        }
        if name.starts_with("USAGE_") {
            name = name.replace("USAGE_", "")
        }
        if name.starts_with("FORMAT_") {
            name = name.replace("FORMAT_", "")
        }
        return Some(name);
    }

    fn item_name(&self, name: &str) -> Option<String> {
        if name == "EGL_FALSE" || name == "EGL_TRUE" {
            return None
        }
        return Some(name.replace("_", ""));
    }

    fn header_file(&self, _filename: &str) {}

    fn include_file(&self, _filename: &str) {}

    fn read_env_var(&self, _key: &str) {}

    fn blocklisted_type_implements_trait(
        &self,
        _name: &str,
        _derive_trait: bindgen::callbacks::DeriveTrait,
    ) -> Option<bindgen::callbacks::ImplementsTrait> {
        None
    }

    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        vec![]
    }

    fn process_comment(&self, _comment: &str) -> Option<String> {
        None
    }

    fn field_visibility(
        &self,
        _info: bindgen::callbacks::FieldInfo<'_>,
    ) -> Option<bindgen::FieldVisibilityKind> {
        None
    }
}

pub fn main() {
    println!("cargo:rerun-if-changed=hardware_buffer.h");
    bindgen::Builder::default()
    .header("hardware_buffer.h")
    .disable_name_namespacing()
    .newtype_enum("AHardwareBuffer_Format")
    .bitfield_enum("AHardwareBuffer_UsageFlags")
    .allowlist_item("ARect")
    .allowlist_item("AHardwareBuffer_Format")
    .allowlist_item("AHardwareBuffer_UsageFlags")
    .allowlist_item("AHardwareBuffer_Desc")
    .allowlist_item("AHardwareBuffer_Plane")
    .allowlist_item("AHardwareBuffer_Planes")
    .allowlist_item("AHardwareBuffer")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .parse_callbacks(Box::new(EnumRenameCallbacks))
    .generate().unwrap()
    .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("hardware_buffer.rs")).unwrap();
}
