use clap::Parser;

use std::{fs, path::PathBuf};

use self::{generate::*, schema::*, types::*};

mod generate;
mod schema;
mod types;

/// xsd-generator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// XSD file to read
    input: PathBuf,

    /// Rust file to write
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let xml_content = fs::read_to_string(args.input)?;
    let xsd_schema: XsdSchema = quick_xml::de::from_str(&xml_content)?;

    let mut buf = String::new();

    buf.push_str("use serde::{Deserialize, Serialize};");

    for ty in xsd_schema.complex_types {
        generate_complex(ty, &mut buf);
    }

    for ty in xsd_schema.simple_types {
        generate_simple(ty, &mut buf);
    }

    for elem in xsd_schema.elements {
        generate_element(elem, &mut buf);
    }

    // println!("{buf}");
    fs::write(args.output, buf)?;

    Ok(())
}
