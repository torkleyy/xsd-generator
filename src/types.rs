use crate::{XsdAttribute, XsdElement, XsdUse};

impl XsdElement {
    pub fn rs_type_name(&self) -> String {
        let base_name = if self.complex_type.is_some() || self.simple_type.is_some() {
            to_pascal_case(&self.name)
        } else {
            map_to_rust_type(
                self.data_type
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or("xs:string"),
            )
        };

        if let Some(max_occurs) = &self.max_occurs {
            if max_occurs != "1" {
                return format!("Vec<{base_name}>");
            }
        } else if let Some(min_occurs) = self.min_occurs {
            if min_occurs == 0 {
                return format!("Option<{base_name}>");
            }
        }

        base_name
    }
}

impl XsdAttribute {
    pub fn rs_type_name(&self, prefix: &str) -> String {
        let base_name = if self.complex_type.is_some() || self.simple_type.is_some() {
            let name = to_pascal_case(&self.name);

            format!("{prefix}{name}")
        } else {
            map_to_rust_type(self.data_type.as_ref().unwrap())
        };

        if matches!(self.use_, XsdUse::Optional) {
            return format!("Option<{base_name}>");
        }

        base_name
    }
}

pub fn map_to_rust_type(xsd_type: &str) -> String {
    if let Some(ty) = xsd_type
        .strip_prefix("xs:")
        .or_else(|| xsd_type.strip_prefix("xsd:"))
    {
        map_primitive_to_rust_type(ty).to_owned()
    } else if !xsd_type.is_empty() {
        to_pascal_case(xsd_type)
    } else {
        "String".to_owned()
    }
}

/// assumes no prefix
pub fn map_primitive_to_rust_type(prim_type: &str) -> &'static str {
    match prim_type {
        "string" | "normalizedString" | "token" | "language" | "Name" | "NMTOKEN" => "String",
        "boolean" => "bool",
        "decimal" => "f64",
        "float" => "f32",
        "double" => "f64",
        "integer" => "i64",
        "nonPositiveInteger" => "i64",
        "negativeInteger" => "i64",
        "long" => "i64",
        "int" => "i32",
        "short" => "i16",
        "byte" => "i8",
        "nonNegativeInteger" => "u64",
        "unsignedLong" => "u64",
        "unsignedInt" => "u32",
        "unsignedShort" => "u16",
        "unsignedByte" => "u8",
        "positiveInteger" => "u64",
        "dateTime" | "time" | "date" | "gYearMonth" | "gYear" | "gMonthDay" | "gDay" | "gMonth"
        | "duration" => "String", // might change to chrono type
        _ => "String", // Default to String for unknown types
    }
}

pub fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    if input.contains('_')
        || input.contains('-')
        || input
            .chars()
            .all(|c| !c.is_alphabetic() || c.is_ascii_uppercase())
    {
        for c in input.chars() {
            if c == '_' || c == '-' {
                capitalize_next = true;
            } else if c.is_ascii_alphanumeric() {
                if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c.to_ascii_lowercase());
                }
            }
        }
        result = to_pascal_case(&result);
    } else {
        result = input.to_owned();
    }

    result
}

pub fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char = '\0';

    for c in input.chars() {
        if c.is_ascii_uppercase() {
            if !prev_char.is_ascii_uppercase() && prev_char != '\0' {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c);
        }
        prev_char = c;
    }

    // TODO: other keywords
    if result == "type" {
        result.insert_str(0, "r#");
    }

    result
}
