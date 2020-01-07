use quick_xml::de::from_str;
use quick_xml::de::DeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Opml {
    version: String,
    head: Head,
    body: Body,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Head {
    title: Option<String>,
    date_created: Option<String>,
    date_modified: Option<String>,
    owner_name: Option<String>,
    owner_email: Option<String>,
    owner_id: Option<String>,
    docs: Option<String>,
    expansion_state: Option<String>,
    vert_scroll_state: Option<String>,
    window_top: Option<String>,
    window_bottom: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Body {
    #[serde(rename = "outline")]
    outlines: Vec<Outline>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    text: String,
    #[serde(rename = "outline")]
    outlines: Option<Vec<Outline>>,
    #[serde(rename = "type")]
    type_: Option<String>,
    xml_url: Option<String>,
    description: Option<String>,
    html_url: Option<String>,
    title: Option<String>,
    version: Option<String>,
    language: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

pub fn parse(xml: &str) -> Result<Opml, OpmlError> {
    Ok(from_str(xml).map_err(OpmlError::XmlError)?)
}

impl Opml {
    /// Get a vec of xml_urls for each <outline type=rss> element.
    pub fn get_xml_urls(&self) -> Result<Vec<String>, OpmlError> {
        let mut res: Vec<String> = Vec::new();
        let acc: &mut Vec<Outline> = &mut Vec::new();
        for o in &self.body.outlines {
            flatten(&o, acc);
        }
        for o in acc {
            if let Some(t) = &o.type_ {
                if t == "rss" {
                    res.push(
                        o.xml_url
                            .as_ref()
                            .ok_or_else(|| OpmlError::BadRss("missing xml_url".into()))?
                            .into(),
                    );
                }
            }
        }
        Ok(res)
    }
}

fn flatten(input: &Outline, mut acc: &mut Vec<Outline>) {
    acc.push(input.clone());
    if let Some(ref children) = input.outlines {
        for child in children {
            flatten(child, &mut acc);
        }
    }
}

#[derive(Debug, Error)]
pub enum OpmlError {
    #[error("Failed to parse outline as rss item: {0}")]
    BadRss(String),
    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] DeError),
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
