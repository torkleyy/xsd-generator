use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdSchema {
    #[serde(rename = "element")]
    #[serde(default)]
    pub elements: Vec<XsdElement>,
    #[serde(rename = "complexType")]
    #[serde(default)]
    pub complex_types: Vec<XsdComplexType>,
    #[serde(rename = "simpleType")]
    #[serde(default)]
    pub simple_types: Vec<XsdSimpleType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdElement {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub data_type: Option<String>,
    #[serde(rename = "complexType")]
    pub complex_type: Option<XsdComplexType>,
    #[serde(rename = "simpleType")]
    pub simple_type: Option<XsdSimpleType>,
    #[serde(rename = "@minOccurs")]
    pub min_occurs: Option<i64>,
    #[serde(rename = "@maxOccurs")]
    pub max_occurs: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdComplexType {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "sequence")]
    pub sequence: Option<Sequence>,
    #[serde(rename = "choice")]
    pub choice: Option<Choice>,
    #[serde(rename = "all")]
    pub all: Option<All>,
    #[serde(rename = "attribute")]
    #[serde(default)]
    pub attributes: Vec<XsdAttribute>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdAttribute {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub data_type: Option<String>,
    #[serde(rename = "@use")]
    pub use_: XsdUse,
    #[serde(rename = "complexType")]
    pub complex_type: Option<XsdComplexType>,
    #[serde(rename = "simpleType")]
    pub simple_type: Option<XsdSimpleType>,
}

#[derive(Debug, Deserialize)]
pub enum XsdUse {
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "prohibited")]
    Prohibited,
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Deserialize)]
pub struct Sequence {
    #[serde(rename = "element")]
    pub elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Choice {
    #[serde(rename = "element")]
    pub elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
pub struct All {
    #[serde(rename = "element")]
    pub elements: Vec<XsdElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdSimpleType {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    pub restriction: Option<Restriction>,
}

#[derive(Debug, Deserialize)]
pub struct Restriction {
    #[serde(rename = "@base")]
    pub base: String,
    pub enumeration: Option<Vec<Enumeration>>,
}

#[derive(Debug, Deserialize)]
pub struct Enumeration {
    #[serde(rename = "@value")]
    pub value: String,
}
