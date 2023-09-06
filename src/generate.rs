use std::fmt::Write;

use crate::*;

pub fn generate_element(elem: XsdElement, f: &mut String) {
    if let Some(mut ty) = elem.complex_type {
        ty.name = Some(elem.name.clone());

        generate_complex(ty, f);
    } else if let Some(mut ty) = elem.simple_type {
        ty.name = Some(elem.name.clone());

        generate_simple(ty, f);
    }
}

pub fn generate_complex(ty: XsdComplexType, f: &mut String) {
    let name = to_pascal_case(&ty.name.unwrap());

    if let Some(mut all) = ty.all {
        if let Some(tl_min) = all.min_occurs {
            all.elements.iter_mut().for_each(|el| {
                el.min_occurs = Some(
                    el.min_occurs
                        .map(|el_min| tl_min * el_min)
                        .unwrap_or(tl_min),
                )
            })
        }

        generate_struct(&name, all.elements, ty.attributes, None, f);
    } else if let Some(_choice) = ty.choice {
        unimplemented!()
    } else if let Some(mut seq) = ty.sequence {
        if let Some(tl_min) = seq.min_occurs {
            seq.elements.iter_mut().for_each(|el| {
                el.min_occurs = Some(
                    el.min_occurs
                        .map(|el_min| tl_min * el_min)
                        .unwrap_or(tl_min),
                )
            })
        }

        generate_struct(&name, seq.elements, ty.attributes, None, f);
    } else if let Some(simple) = ty.simple_content {
        generate_struct(
            &name,
            vec![],
            simple.extension.attributes,
            Some(simple.extension.base),
            f,
        );
    }
}

pub fn generate_attribute(prefix: &str, attr: XsdAttribute, f: &mut String) {
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

pub fn generate_simple(ty: XsdSimpleType, f: &mut String) {
    let xml_ty_name = ty.name.unwrap();
    let name = to_pascal_case(&xml_ty_name);

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
            // we need to create a wrapper struct
            // see https://docs.rs/quick-xml/latest/quick_xml/de/#enumunit-variants-as-a-text
            let snake_name = to_snake_case(&name);
            let _ = writeln!(f, "#[derive(Clone, Debug, Deserialize, Serialize)]");
            let _ = writeln!(f, "pub struct {name} {{");
            let _ = writeln!(f, "#[serde(rename = \"{xml_ty_name}\")]");
            let _ = writeln!(f, "pub {snake_name}: {name}Enum,");
            let _ = writeln!(f, "}}");

            let rs_type = name;
            let _ = writeln!(f, "#[derive(Clone, Debug, Deserialize, Serialize)]");
            let _ = writeln!(f, "pub enum {rs_type}Enum {{");
            for v in en {
                let xml_name = &v.value;
                let v_ident = to_pascal_case(xml_name);
                let _ = writeln!(f, "#[serde(rename = \"{xml_name}\")]");
                let _ = writeln!(f, "{v_ident},");
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

pub fn generate_struct(
    name: &str,
    elements: Vec<XsdElement>,
    attribs: Vec<XsdAttribute>,
    content: Option<String>,
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

    if let Some(content) = &content {
        let rs_type = map_primitive_to_rust_type(content);
        let _ = writeln!(f, "#[serde(rename = \"$value\")]");
        let _ = writeln!(f, "pub value: {rs_type},");
    }

    let _ = writeln!(f, "}}");

    for attr in attribs {
        generate_attribute(name, attr, f);
    }

    for elem in elements {
        generate_element(elem, f);
    }
}
