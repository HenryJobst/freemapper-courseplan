//! Copied from freemapper-print 8b0a0a6 (ADR 0017).
//! Symbol sets: colours and symbols copied verbatim from OOM symbol set files.

use super::OmapError;

/// Symbol number of a norm, e.g. "521" or "501.5".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolCode(pub String);

/// Colours, symbols and grid of a symbol set as raw OOM XML fragments plus a code → id table.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolSet {
    /// Value of `<symbols id="…">`, e.g. "ISSprOM 2019".
    pub name: String,
    pub scale_denominator: u32,
    pub colors_xml: String,
    pub symbols_xml: String,
    pub grid_xml: String,
    codes: Vec<(SymbolCode, u32)>,
    hidden: Vec<u32>,
}

impl SymbolSet {
    /// Extracts colours, symbols, grid and the code → id table from an OOM map file.
    pub fn from_omap_xml(xml: &str) -> Result<Self, OmapError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        let mut stack: Vec<String> = Vec::new();
        let mut scale_denominator = None;
        let mut name = None;
        let (mut colors, mut symbols, mut grid) = (None, None, None);
        let (mut colors_start, mut symbols_start) = (0, 0);
        let mut codes = Vec::new();
        let mut hidden = Vec::new();

        loop {
            let before = position(&reader);
            let event = reader
                .read_event()
                .map_err(|e| OmapError::Xml(e.to_string()))?;
            let after = position(&reader);
            let (e, is_start) = match event {
                Event::Start(e) => (e, true),
                Event::Empty(e) => (e, false),
                Event::End(e) => {
                    match e.name().as_ref() {
                        "colors" => colors = Some(xml[colors_start..after].to_string()),
                        "symbols" => symbols = Some(xml[symbols_start..after].to_string()),
                        _ => {}
                    }
                    stack.pop();
                    continue;
                }
                Event::Eof => break,
                _ => continue,
            };
            let tag = e.name().as_ref().to_string();
            let parent = stack.last().map(String::as_str);
            match (tag.as_str(), parent) {
                ("georeferencing", _) => {
                    scale_denominator = attribute(&e, "scale")?.and_then(|v| v.parse().ok());
                }
                ("colors", _) => colors_start = before,
                ("symbols", _) => {
                    symbols_start = before;
                    name = attribute(&e, "id")?;
                }
                ("symbol", Some("symbols")) => {
                    let id = attribute(&e, "id")?.and_then(|v| v.parse::<u32>().ok());
                    let code = attribute(&e, "code")?;
                    if let (Some(id), Some("true")) = (id, attribute(&e, "is_hidden")?.as_deref()) {
                        hidden.push(id);
                    }
                    if let (Some(id), Some(code)) = (id, code) {
                        codes.push((SymbolCode(code), id));
                    }
                }
                ("grid", Some("view")) => grid = Some(xml[before..after].to_string()),
                _ => {}
            }
            if is_start {
                stack.push(tag);
            }
        }

        Ok(Self {
            name: name.ok_or(OmapError::MissingElement("symbols"))?,
            scale_denominator: scale_denominator
                .ok_or(OmapError::MissingElement("georeferencing"))?,
            colors_xml: colors.ok_or(OmapError::MissingElement("colors"))?,
            symbols_xml: symbols.ok_or(OmapError::MissingElement("symbols"))?,
            grid_xml: grid.unwrap_or_default(),
            codes,
            hidden,
        })
    }

    /// Whether the symbol is marked `is_hidden="true"` in the file (not drawn by OOM).
    pub fn is_hidden(&self, id: u32) -> bool {
        self.hidden.contains(&id)
    }

    /// Code of the top-level symbol with the given id.
    pub fn code_for_id(&self, id: u32) -> Option<&SymbolCode> {
        self.codes.iter().find(|(_, i)| *i == id).map(|(c, _)| c)
    }

    /// Id of the first top-level symbol with the given code.
    pub fn id_for_code(&self, code: &str) -> Option<u32> {
        self.codes
            .iter()
            .find(|(c, _)| c.0 == code)
            .map(|(_, id)| *id)
    }

    /// OOM type of the top-level symbol `id` (1 point, 2 line, 4 area, 8 text, 16 combined).
    pub fn symbol_type(&self, id: u32) -> Option<u32> {
        let marker = format!(r#"" id="{id}""#);
        self.symbols_xml
            .match_indices(r#"<symbol type=""#)
            .find_map(|(at, open)| {
                let rest = &self.symbols_xml[at + open.len()..];
                let digits = rest.find('"')?;
                rest[digits..]
                    .starts_with(&marker)
                    .then(|| rest[..digits].parse().ok())?
            })
    }
}

/// Current byte offset of `reader` in its input.
pub(crate) fn position(reader: &quick_xml::Reader<&[u8]>) -> usize {
    usize::try_from(reader.buffer_position()).expect("XML input fits in memory")
}

/// Unescaped value of attribute `name`, if present.
pub(crate) fn attribute(
    e: &quick_xml::events::BytesStart<'_>,
    name: &str,
) -> Result<Option<String>, OmapError> {
    let attr = e
        .try_get_attribute(name)
        .map_err(|err| OmapError::Xml(err.to_string()))?;
    attr.map(|a| {
        a.normalized_value(quick_xml::XmlVersion::Explicit1_0)
            .map(|v| v.into_owned())
    })
    .transpose()
    .map_err(|err| OmapError::Xml(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<map><georeferencing scale="4000"/><colors count="0"></colors><symbols count="3" id="Test"><symbol type="1" id="0" code="101" name="a"><point_symbol/></symbol><symbol type="2" id="2" code="501.5" name="b" is_hidden="true"><line_symbol/></symbol><symbol type="4" id="5" code="521" name="c"><area_symbol/></symbol></symbols><view><grid/></view></map>"#;

    #[test]
    fn codes_types_and_hidden() {
        let set = SymbolSet::from_omap_xml(XML).unwrap();
        assert_eq!(set.name, "Test");
        assert_eq!(set.scale_denominator, 4000);
        assert_eq!(set.id_for_code("501.5"), Some(2));
        assert_eq!(set.id_for_code("9999"), None);
        assert_eq!(set.symbol_type(0), Some(1));
        assert_eq!(set.symbol_type(2), Some(2));
        assert_eq!(set.symbol_type(5), Some(4));
        assert_eq!(set.symbol_type(7), None);
        assert!(set.is_hidden(2));
        assert!(!set.is_hidden(0));
    }

    #[test]
    fn code_for_id_inverts_id_for_code() {
        let set = SymbolSet::from_omap_xml(XML).unwrap();
        assert_eq!(set.code_for_id(5), Some(&SymbolCode("521".into())));
        assert_eq!(set.code_for_id(1), None);
    }
}
