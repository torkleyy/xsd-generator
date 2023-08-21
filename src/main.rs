use serde::Deserialize;
use std::{fmt::Write, fs};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdSchema {
    #[serde(rename = "element")]
    #[serde(default)]
    elements: Vec<XsdElement>,
    #[serde(rename = "complexType")]
    #[serde(default)]
    complex_types: Vec<XsdComplexType>,
    #[serde(rename = "simpleType")]
    #[serde(default)]
    simple_types: Vec<XsdSimpleType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdElement {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    data_type: Option<String>,
    #[serde(rename = "complexType")]
    complex_type: Option<XsdComplexType>,
    #[serde(rename = "simpleType")]
    simple_type: Option<XsdSimpleType>,
    #[serde(rename = "@minOccurs")]
    min_occurs: Option<i64>,
    #[serde(rename = "@maxOccurs")]
    max_occurs: Option<String>,
}

impl XsdElement {
    pub fn rs_type_name(&self) -> String {
        let base_name = if self.complex_type.is_some() || self.simple_type.is_some() {
            to_pascal_case(&self.name)
        } else {
            map_to_rust_type(self.data_type.as_ref().unwrap())
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdComplexType {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde(rename = "sequence")]
    sequence: Option<Sequence>,
    #[serde(rename = "choice")]
    choice: Option<Choice>,
    #[serde(rename = "all")]
    all: Option<All>,
    #[serde(rename = "attribute")]
    #[serde(default)]
    attributes: Vec<XsdAttribute>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdAttribute {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    data_type: Option<String>,
    #[serde(rename = "@use")]
    use_: XsdUse,
    #[serde(rename = "complexType")]
    complex_type: Option<XsdComplexType>,
    #[serde(rename = "simpleType")]
    simple_type: Option<XsdSimpleType>,
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

#[derive(Debug, Deserialize)]
enum XsdUse {
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "prohibited")]
    Prohibited,
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Deserialize)]
struct Sequence {
    #[serde(rename = "element")]
    elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Choice {
    #[serde(rename = "element")]
    elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
struct All {
    #[serde(rename = "element")]
    elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdSimpleType {
    #[serde(rename = "@name")]
    name: Option<String>,
    restriction: Option<Restriction>,
}

#[derive(Debug, Deserialize)]
struct Restriction {
    #[serde(rename = "@base")]
    base: String,
    enumeration: Option<Vec<Enumeration>>,
}

#[derive(Debug, Deserialize)]
struct Enumeration {
    #[serde(rename = "@value")]
    value: String,
}

fn generate_element(elem: XsdElement, f: &mut String) {
    if let Some(mut ty) = elem.complex_type {
        ty.name = Some(elem.name.clone());

        generate_complex(ty, f);
    } else if let Some(mut ty) = elem.simple_type {
        ty.name = Some(elem.name.clone());

        generate_simple(ty, f);
    }
}

fn generate_complex(ty: XsdComplexType, f: &mut String) {
    let name = to_pascal_case(&ty.name.unwrap());

    if let Some(all) = ty.all {
        generate_struct(&name, all.elements, ty.attributes, f);
    } else if let Some(_choice) = ty.choice {
        unimplemented!()
    } else if let Some(seq) = ty.sequence {
        generate_struct(&name, seq.elements, ty.attributes, f);
    }
}

fn generate_attribute(prefix: &str, attr: XsdAttribute, f: &mut String) {
    let name = &attr.name;
    let name = format!("{prefix}_{name}");
    if let Some(mut ty) = attr.complex_type {
        ty.name = Some(name);

        generate_complex(ty, f);
    } else if let Some(mut ty) = attr.simple_type {
        ty.name = Some(name);

        generate_simple(ty, f);
    }
}

fn generate_simple(ty: XsdSimpleType, f: &mut String) {
    let name = to_pascal_case(&ty.name.unwrap());

    if let Some(restr) = ty.restriction {
        if let Some(en) = restr.enumeration.filter(|en| {
            en.iter().all(|v| {
                v.value
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_alphabetic())
                    .unwrap_or(false)
            })
        }) {
            let rs_type = map_to_rust_type(&restr.base);
            let _ = writeln!(f, "#[derive(Clone, Debug, Deserialize, Serialize)]");
            let _ = writeln!(f, "pub enum {rs_type} {{");
            for v in en {
                let xml_name = &v.value;
                let v_ident = to_pascal_case(xml_name);
                let _ = writeln!(f, "#[serde(rename = \"{xml_name}\")]");
                let _ = writeln!(f, "{v_ident}");
            }
            let _ = writeln!(f, "}}");
        } else {
            let rs_type = map_to_rust_type(&restr.base);
            let _ = writeln!(f, "pub type {name} = {rs_type};");
        }
    } else {
        let _ = writeln!(f, "pub type {name} = String;");
    }
}

fn generate_struct(
    name: &str,
    elements: Vec<XsdElement>,
    attribs: Vec<XsdAttribute>,
    f: &mut String,
) {
    let _ = writeln!(f, "#[derive(Clone, Debug, Deserialize, Serialize)]");
    let _ = writeln!(f, "pub struct {name} {{");

    for attr in &attribs {
        if matches!(attr.use_, XsdUse::Prohibited) {
            continue;
        }

        let xml_name = &attr.name;
        let rs_name = to_snake_case(&attr.name);
        let ty_name = attr.rs_type_name(name);
        let _ = writeln!(f, "#[serde(rename = \"@{xml_name}\")]");
        let _ = writeln!(f, "pub {rs_name}: {ty_name},");
    }

    for elem in &elements {
        let xml_name = &elem.name;
        let rs_name = to_snake_case(&elem.name);
        let ty_name = elem.rs_type_name();
        if let Some(min_occurs) = elem.min_occurs {
            if min_occurs == 0 {
                let _ = writeln!(f, "#[serde(default)]");
            }
        }
        let _ = writeln!(f, "#[serde(rename = \"{xml_name}\")]");
        let _ = writeln!(f, "pub {rs_name}: {ty_name},");
    }

    let _ = writeln!(f, "}}");

    for attr in attribs {
        generate_attribute(name, attr, f);
    }

    for elem in elements {
        generate_element(elem, f);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string("schema.xsd")?;
    let xsd_schema: XsdSchema = quick_xml::de::from_str(&xml_content)?;

    let mut buf = String::new();

    for ty in xsd_schema.complex_types {
        generate_complex(ty, &mut buf);
    }

    for ty in xsd_schema.simple_types {
        generate_simple(ty, &mut buf);
    }

    for elem in xsd_schema.elements {
        generate_element(elem, &mut buf);
    }

    println!("{buf}");

    Ok(())
}

fn map_to_rust_type(xsd_type: &str) -> String {
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
fn map_primitive_to_rust_type(prim_type: &str) -> &'static str {
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

fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in input.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if c.is_ascii_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
    }

    result
}

fn to_snake_case(input: &str) -> String {
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
