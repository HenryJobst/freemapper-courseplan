//! The sync base copy: where it lives, its hash, and what is stripped from it (D1, D2).

use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use freemapper_courseplan_model::SyncState;

use super::apply::SyncNote;

/// FNV-1a 64 over `bytes`, as 16 lowercase hex digits. It detects drift between files and is not a
/// security measure (D2).
pub fn fnv1a64(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The base copy that belongs to a project file (D1): the file name with `.toml` replaced by
/// `.ppen-base`, or with `.ppen-base` appended when it does not end in `.toml`.
pub fn base_path(project: &Path) -> PathBuf {
    if project.extension().is_some_and(|e| e == "toml") {
        project.with_extension("ppen-base")
    } else {
        let mut name: OsString = project.as_os_str().to_owned();
        name.push(".ppen-base");
        PathBuf::from(name)
    }
}

/// Removes the `absolute-path` attribute of the `<map>` start element and nothing else (D1 ruling).
///
/// The attribute holds the author's home path and is not data the merge compares, so the base copy does
/// not keep it. The edit is textual: the attribute and the white space before it go, every other byte
/// (BOM, line endings, quoting, order) stays. Without the attribute, or when the start element cannot be
/// read, the bytes come back unchanged.
///
/// The `base_fnv1a64` in `[sync]` is the hash of these stripped bytes, not of the `.ppen` as Purple Pen
/// wrote it.
pub fn strip_absolute_path(bytes: &[u8]) -> Vec<u8> {
    match find_attribute(bytes) {
        Some((from, to)) => [&bytes[..from], &bytes[to..]].concat(),
        None => bytes.to_vec(),
    }
}

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}

/// The byte range of the white space before `absolute-path` through the closing quote of its value.
fn find_attribute(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
        } else if bytes[i..].starts_with(b"<!--") {
            i = skip_past(bytes, i + 4, b"-->")?;
        } else if bytes[i..].starts_with(b"<![CDATA[") {
            i = skip_past(bytes, i + 9, b"]]>")?;
        } else if bytes[i..].starts_with(b"<map")
            && bytes
                .get(i + 4)
                .is_some_and(|&b| is_space(b) || b == b'>' || b == b'/')
        {
            return attribute_in_tag(bytes, i + 4);
        } else {
            i += 1;
        }
    }
    None
}

fn skip_past(bytes: &[u8], from: usize, end: &[u8]) -> Option<usize> {
    bytes[from..]
        .windows(end.len())
        .position(|w| w == end)
        .map(|p| from + p + end.len())
}

/// Walks the attributes of a start tag whose name ends at `i`.
fn attribute_in_tag(bytes: &[u8], mut i: usize) -> Option<(usize, usize)> {
    loop {
        let space_from = i;
        while bytes.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        match bytes.get(i)? {
            b'>' | b'/' => return None,
            _ => {}
        }
        let name_from = i;
        while bytes
            .get(i)
            .is_some_and(|&b| !is_space(b) && !matches!(b, b'=' | b'>' | b'/'))
        {
            i += 1;
        }
        let name = &bytes[name_from..i];
        while bytes.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        if *bytes.get(i)? != b'=' || name.is_empty() {
            return None;
        }
        i += 1;
        while bytes.get(i).is_some_and(|&b| is_space(b)) {
            i += 1;
        }
        let quote = *bytes.get(i)?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        let close = bytes[i + 1..].iter().position(|&b| b == quote)? + i + 1;
        i = close + 1;
        if name == b"absolute-path" {
            // Only the white space directly before the attribute goes with it.
            return (space_from < name_from).then_some((space_from, i));
        }
    }
}

/// What to do with the stored base copy before a sync (D13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseState {
    /// The base copy is the one the last sync left: use it.
    Use,
    /// The last sync stopped after it wrote the project: the recorded hash is the linked file's, so the
    /// linked file is the base.
    Repair,
    /// The first sync: the project has no `[sync]` yet and no base copy exists. Nothing is wrong; the
    /// sync runs without a base, as with `Ignore`, but says so neutrally.
    First,
    /// Sync without a base. `why` is `"missing"`, `"other link"` or `"hash"`.
    Ignore(&'static str),
}

/// Judges the stored base copy (D13). `theirs` is the linked file as it is now; the hashes cover the bytes
/// without `absolute-path` ([`strip_absolute_path`]).
///
/// A base is used only when the project records it (`sync.ppen` is this link and `base_fnv1a64` is the hash
/// of the stored bytes). The recorded hash of the linked file itself means that only the base write is
/// missing. Anything else, a base that does not belong to this project among it, is ignored, so that it
/// can never cause a deletion.
pub fn base_state(
    sync: Option<&SyncState>,
    link: &Path,
    base: Option<&[u8]>,
    theirs: &[u8],
) -> BaseState {
    let Some(sync) = sync else {
        return if base.is_none() {
            BaseState::First
        } else {
            BaseState::Ignore("missing")
        };
    };
    if sync.ppen.as_deref() != Some(link) {
        return BaseState::Ignore("other link");
    }
    let (Some(base), Some(recorded)) = (base, sync.base_fnv1a64.as_deref()) else {
        return BaseState::Ignore("missing");
    };
    if fnv1a64(base) == recorded {
        BaseState::Use
    } else if fnv1a64(&strip_absolute_path(theirs)) == recorded {
        BaseState::Repair
    } else {
        BaseState::Ignore("hash")
    }
}

impl BaseState {
    /// The base bytes for [`SyncInput::base`](crate::sync::SyncInput) and the note to put in
    /// `base_note`: the stored copy for `Use`, the stripped linked file for `Repair`, nothing for `Ignore`.
    pub fn resolve<'a>(
        self,
        base: Option<&'a [u8]>,
        theirs: &[u8],
    ) -> (Option<Cow<'a, [u8]>>, Option<SyncNote>) {
        match self {
            BaseState::Use => (base.map(Cow::Borrowed), None),
            BaseState::Repair => (
                Some(Cow::Owned(strip_absolute_path(theirs))),
                Some(SyncNote::BaseRepaired),
            ),
            BaseState::First => (None, Some(SyncNote::FirstSync)),
            BaseState::Ignore(why) => (None, Some(SyncNote::BaseIgnored { why })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(s: &str) -> String {
        String::from_utf8(strip_absolute_path(s.as_bytes())).unwrap()
    }

    #[test]
    fn fnv_of_nothing_is_the_offset_basis() {
        assert_eq!(fnv1a64(b""), "cbf29ce484222325");
        assert_eq!(fnv1a64(b"a"), "af63dc4c8601ec8c");
    }

    #[test]
    fn base_path_replaces_toml_or_appends() {
        assert_eq!(
            base_path(Path::new("e/x.courseplan.toml")),
            PathBuf::from("e/x.courseplan.ppen-base")
        );
        assert_eq!(
            base_path(Path::new("e/x.cp")),
            PathBuf::from("e/x.cp.ppen-base")
        );
        assert_eq!(
            base_path(Path::new("e/.toml")),
            PathBuf::from("e/.toml.ppen-base")
        );
    }

    #[test]
    fn strips_the_attribute_in_the_middle_first_and_last() {
        assert_eq!(
            strip(r#"<a><map a="1" absolute-path="/h/m.omap" b="2"/></a>"#),
            r#"<a><map a="1" b="2"/></a>"#
        );
        assert_eq!(
            strip(r#"<map absolute-path="/h/m.omap" b="2"/>"#),
            r#"<map b="2"/>"#
        );
        assert_eq!(
            strip(r#"<map a="1" absolute-path='/h'>x</map>"#),
            r#"<map a="1">x</map>"#
        );
        assert_eq!(strip(r#"<map absolute-path="/h"/>"#), "<map/>");
    }

    #[test]
    fn keeps_everything_else_byte_for_byte() {
        let text = "\u{feff}<?xml version=\"1.0\"?>\r\n<event>\r\n  <map scale=\"10000\"\r\n       absolute-path = \"C:\\Users\\me\\m.omap\"\r\n       relative-path=\"m.omap\" />\r\n</event>\r\n";
        assert_eq!(
            strip(text),
            "\u{feff}<?xml version=\"1.0\"?>\r\n<event>\r\n  <map scale=\"10000\"\r\n       relative-path=\"m.omap\" />\r\n</event>\r\n"
        );
    }

    #[test]
    fn leaves_other_text_alone() {
        // Absent, in another element, in a value, in a comment, or a longer element name.
        for text in [
            r#"<map a="1"/>"#,
            r#"<other absolute-path="x"/>"#,
            r#"<mapper absolute-path="x"/>"#,
            r#"<map a=" absolute-path='x' "/>"#,
            r#"<!-- <map absolute-path="x"/> --><map a="1"/>"#,
            r#"<map a="1""#,
            "",
        ] {
            assert_eq!(strip(text), text);
        }
        assert_eq!(
            strip(r#"<!-- <map absolute-path="x"/> --><map absolute-path="y" a="1"/>"#),
            r#"<!-- <map absolute-path="x"/> --><map a="1"/>"#
        );
        assert_eq!(
            strip(r#"<map a=">" absolute-path="/x>y"/>"#),
            r#"<map a=">"/>"#
        );
    }

    #[test]
    fn is_idempotent() {
        let once = strip(r#"<map absolute-path="/h" b="2"/>"#);
        assert_eq!(strip(&once), once);
    }

    fn state(link: &str, base: &[u8]) -> SyncState {
        SyncState {
            ppen: Some(PathBuf::from(link)),
            base_fnv1a64: Some(fnv1a64(base)),
            decisions: vec![],
        }
    }

    #[test]
    fn matching_hash_is_used() {
        let s = state("a.ppen", b"base");
        assert_eq!(
            base_state(Some(&s), Path::new("a.ppen"), Some(b"base"), b"theirs"),
            BaseState::Use
        );
    }

    #[test]
    fn recorded_hash_of_theirs_repairs() {
        let s = state("a.ppen", b"theirs");
        assert_eq!(
            base_state(Some(&s), Path::new("a.ppen"), Some(b"old"), b"theirs"),
            BaseState::Repair
        );
        // The hash covers the stripped bytes.
        let theirs = br#"<map absolute-path="/h" a="1"/>"#;
        let s = state("a.ppen", &strip_absolute_path(theirs));
        assert_eq!(
            base_state(Some(&s), Path::new("a.ppen"), Some(b"old"), theirs),
            BaseState::Repair
        );
        let (base, note) = BaseState::Repair.resolve(Some(b"old"), theirs);
        assert_eq!(base.unwrap().as_ref(), br#"<map a="1"/>"#);
        assert_eq!(note, Some(SyncNote::BaseRepaired));
    }

    #[test]
    fn foreign_base_is_ignored() {
        let s = state("a.ppen", b"recorded");
        assert_eq!(
            base_state(Some(&s), Path::new("a.ppen"), Some(b"foreign"), b"theirs"),
            BaseState::Ignore("hash")
        );
    }

    #[test]
    fn other_link_is_ignored() {
        let s = state("a.ppen", b"base");
        assert_eq!(
            base_state(Some(&s), Path::new("b.ppen"), Some(b"base"), b"theirs"),
            BaseState::Ignore("other link")
        );
    }

    #[test]
    fn no_base_file_is_ignored() {
        let s = state("a.ppen", b"base");
        assert_eq!(
            base_state(Some(&s), Path::new("a.ppen"), None, b"theirs"),
            BaseState::Ignore("missing")
        );
        // A base file without any `[sync]` is not a normal first sync.
        assert_eq!(
            base_state(None, Path::new("a.ppen"), Some(b"base"), b"theirs"),
            BaseState::Ignore("missing")
        );
        let no_hash = SyncState {
            ppen: Some(PathBuf::from("a.ppen")),
            ..SyncState::default()
        };
        assert_eq!(
            base_state(Some(&no_hash), Path::new("a.ppen"), Some(b"base"), b"t"),
            BaseState::Ignore("missing")
        );
    }

    #[test]
    fn first_sync_has_its_own_state_and_note() {
        assert_eq!(
            base_state(None, Path::new("a.ppen"), None, b"theirs"),
            BaseState::First
        );
        let (b, n) = BaseState::First.resolve(None, b"t");
        assert!(b.is_none());
        assert_eq!(n, Some(SyncNote::FirstSync));
    }

    #[test]
    fn resolve_gives_base_and_note() {
        let (b, n) = BaseState::Use.resolve(Some(b"x"), b"t");
        assert_eq!(b.unwrap().as_ref(), b"x");
        assert_eq!(n, None);
        let (b, n) = BaseState::Ignore("hash").resolve(Some(b"x"), b"t");
        assert!(b.is_none());
        assert_eq!(n, Some(SyncNote::BaseIgnored { why: "hash" }));
    }
}
