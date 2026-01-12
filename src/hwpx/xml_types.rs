use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HcfVersion {
    #[serde(rename = "@version", default)]
    pub version: Option<String>,
    #[serde(rename = "@xmlVersion", default)]
    pub xml_version: Option<String>,
    #[serde(rename = "@major", default)]
    pub major: Option<String>,
    #[serde(rename = "@minor", default)]
    pub minor: Option<String>,
    #[serde(rename = "@tagetApplication", default)]
    pub taget_application: Option<String>,
    #[serde(rename = "@application", default)]
    pub application: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "head")]
pub struct Head {
    #[serde(rename = "beginNum", default)]
    pub begin_num: Option<BeginNum>,
    #[serde(rename = "refList", default)]
    pub ref_list: Option<RefList>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeginNum {
    #[serde(rename = "@page", default)]
    pub page: Option<u32>,
    #[serde(rename = "@footnote", default)]
    pub footnote: Option<u32>,
    #[serde(rename = "@endnote", default)]
    pub endnote: Option<u32>,
    #[serde(rename = "@pic", default)]
    pub pic: Option<u32>,
    #[serde(rename = "@tbl", default)]
    pub tbl: Option<u32>,
    #[serde(rename = "@equation", default)]
    pub equation: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefList {
    #[serde(rename = "fontfaces", default)]
    pub fontfaces: Option<Fontfaces>,
    #[serde(rename = "charProperties", default)]
    pub char_properties: Option<CharProperties>,
    #[serde(rename = "paraProperties", default)]
    pub para_properties: Option<ParaProperties>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fontfaces {
    #[serde(rename = "fontface", default)]
    pub items: Vec<Fontface>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fontface {
    #[serde(rename = "@lang")]
    pub lang: String,
    #[serde(rename = "font", default)]
    pub fonts: Vec<Font>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Font {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@face")]
    pub face: String,
    #[serde(rename = "@type", default)]
    pub font_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharProperties {
    #[serde(rename = "charPr", default)]
    pub items: Vec<CharPr>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharPr {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@height", default)]
    pub height: Option<u32>,
    #[serde(rename = "@textColor", default)]
    pub text_color: Option<String>,
    #[serde(rename = "@bold", default)]
    pub bold: Option<bool>,
    #[serde(rename = "@italic", default)]
    pub italic: Option<bool>,
    #[serde(rename = "@underline", default)]
    pub underline: Option<String>,
    #[serde(rename = "@strikeout", default)]
    pub strikeout: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParaProperties {
    #[serde(rename = "paraPr", default)]
    pub items: Vec<ParaPr>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParaPr {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@align", default)]
    pub align: Option<String>,
    #[serde(rename = "@lineSpacing", default)]
    pub line_spacing: Option<String>,
    #[serde(rename = "@tabPrIDRef", default)]
    pub tab_pr_id_ref: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "sec")]
pub struct Section {
    #[serde(rename = "p", default)]
    pub paragraphs: Vec<XmlParagraph>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlParagraph {
    #[serde(rename = "@id", default)]
    pub id: Option<u32>,
    #[serde(rename = "@paraPrIDRef", default)]
    pub para_pr_id_ref: Option<u32>,
    #[serde(rename = "@styleIDRef", default)]
    pub style_id_ref: Option<u32>,
    #[serde(rename = "run", default)]
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Run {
    #[serde(rename = "@charPrIDRef", default)]
    pub char_pr_id_ref: Option<u32>,
    #[serde(rename = "t", default)]
    pub text: Option<String>,
    #[serde(rename = "secPr", default)]
    pub sec_pr: Option<SecPr>,
    #[serde(rename = "tbl", default)]
    pub table: Option<XmlTable>,
    #[serde(rename = "pic", default)]
    pub picture: Option<XmlPicture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecPr {
    #[serde(rename = "@textDirection", default)]
    pub text_direction: Option<String>,
    #[serde(rename = "@spaceColumns", default)]
    pub space_columns: Option<u32>,
    #[serde(rename = "pageMargin", default)]
    pub page_margin: Option<PageMargin>,
    #[serde(rename = "pagePr", default)]
    pub page_pr: Option<PagePr>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageMargin {
    #[serde(rename = "@left", default)]
    pub left: Option<u32>,
    #[serde(rename = "@right", default)]
    pub right: Option<u32>,
    #[serde(rename = "@top", default)]
    pub top: Option<u32>,
    #[serde(rename = "@bottom", default)]
    pub bottom: Option<u32>,
    #[serde(rename = "@header", default)]
    pub header: Option<u32>,
    #[serde(rename = "@footer", default)]
    pub footer: Option<u32>,
    #[serde(rename = "@gutter", default)]
    pub gutter: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagePr {
    #[serde(rename = "@width", default)]
    pub width: Option<u32>,
    #[serde(rename = "@height", default)]
    pub height: Option<u32>,
    #[serde(rename = "@landscape", default)]
    pub landscape: Option<String>,
    #[serde(rename = "margin", default)]
    pub margin: Option<PageMargin>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlTable {
    #[serde(rename = "@id", default)]
    pub id: Option<u32>,
    #[serde(rename = "tr", default)]
    pub rows: Vec<XmlTableRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlTableRow {
    #[serde(rename = "tc", default)]
    pub cells: Vec<XmlTableCell>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlTableCell {
    #[serde(rename = "@colSpan", default)]
    pub col_span: Option<u32>,
    #[serde(rename = "@rowSpan", default)]
    pub row_span: Option<u32>,
    #[serde(rename = "cellAddr", default)]
    pub cell_addr: Option<CellAddr>,
    #[serde(rename = "subList", default)]
    pub sub_list: Option<SubList>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CellAddr {
    #[serde(rename = "@colAddr", default)]
    pub col_addr: Option<u32>,
    #[serde(rename = "@rowAddr", default)]
    pub row_addr: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubList {
    #[serde(rename = "p", default)]
    pub paragraphs: Vec<XmlParagraph>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlPicture {
    #[serde(rename = "@id", default)]
    pub id: Option<u32>,
    #[serde(rename = "imgRect", default)]
    pub img_rect: Option<ImgRect>,
    #[serde(rename = "img", default)]
    pub img: Option<Img>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImgRect {
    #[serde(rename = "@x", default)]
    pub x: Option<u32>,
    #[serde(rename = "@y", default)]
    pub y: Option<u32>,
    #[serde(rename = "@cx", default)]
    pub cx: Option<u32>,
    #[serde(rename = "@cy", default)]
    pub cy: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Img {
    #[serde(rename = "@binaryItemIDRef", default)]
    pub binary_item_id_ref: Option<String>,
}

pub fn parse_version(xml: &str) -> Result<HcfVersion, quick_xml::DeError> {
    from_str(xml)
}

pub fn parse_head(xml: &str) -> Result<Head, quick_xml::DeError> {
    from_str(xml)
}

pub fn parse_section(xml: &str) -> Result<Section, quick_xml::DeError> {
    from_str(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><HCFVersion version="1.0"/>"#;
        let result = parse_version(xml);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, Some("1.0".to_string()));
    }

    #[test]
    fn test_parse_simple_section() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sec>
            <p id="0" paraPrIDRef="0">
                <run charPrIDRef="0">
                    <t>Hello World</t>
                </run>
            </p>
        </sec>"#;
        let result = parse_section(xml);
        assert!(result.is_ok());
        let section = result.unwrap();
        assert_eq!(section.paragraphs.len(), 1);
        assert_eq!(section.paragraphs[0].runs.len(), 1);
        assert_eq!(
            section.paragraphs[0].runs[0].text,
            Some("Hello World".to_string())
        );
    }
}
