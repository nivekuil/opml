#[macro_use]
extern crate derive_builder;
use quick_xml::de::from_str;
use quick_xml::de::DeError;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename = "opml")]
pub struct Opml {
    pub version: String,
    pub head: Head,
    pub body: Body,
}

impl Default for Opml {
    fn default() -> Self {
        Opml {
            version: "2.0".into(),
            head: Head::default(),
            body: Body::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Head {
    pub title: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
    pub owner_id: Option<String>,
    pub docs: Option<String>,
    pub expansion_state: Option<String>,
    pub vert_scroll_state: Option<String>,
    pub window_top: Option<String>,
    pub window_bottom: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Body {
    pub outline: Vec<Outline>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Builder, Default)]
#[builder(default)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    text: String,
    outline: Option<Vec<Outline>>,
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
        for o in &self.body.outline {
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

    pub fn to_string(&self) -> Result<String, OpmlError> {
        Ok(to_string(self)?)
    }
}

fn flatten(input: &Outline, mut acc: &mut Vec<Outline>) {
    acc.push(input.clone());
    if let Some(ref children) = input.outline {
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

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        bad("<opml><head></head><body><outline><text></text></outline></body></opml>");
        bad("<head></head><body><outline><text></text></outline></body>");
        ok("<opml version='2.0'><head></head><body><outline><text></text></outline></body></opml>");
        ok("<opml version='x'><head></head><body><outline><text></text></outline></body></opml>");
        bad("<opml version='x'><head></head><body><outline></outline></body></opml>");
        bad("<opml version='x'><head></head><body><outline></outline></body></opml>");
        bad("<opml version='x'><head></head><body></body></opml>");
        bad("<opml version='x'></opml>");
        bad("<opml></opml>");
    }

    #[test]
    fn multi_outline() {
        ok("<opml version='x'><head></head><body><outline><text></text></outline><outline><text></text></outline><outline><text></text></outline></body></opml>");
        ok("<opml version='x'><head></head><body><outline><text></text></outline><outline><text></text><outline><text></text></outline></outline></body></opml>");
    }

    fn bad(xml: &str) {
        assert!(crate::parse(xml).is_err());
    }
    fn ok(xml: &str) {
        assert!(crate::parse(xml).is_ok());
    }
}
