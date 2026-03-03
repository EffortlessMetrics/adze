/// INI file format grammar demonstrating sections, key-value pairs,
/// comments, and repetition with delimiters.
#[adze::grammar("ini_file")]
pub mod grammar {
    use adze::Spanned;

    /// An INI file is a list of entries (sections, pairs, comments, blanks)
    #[adze::language]
    #[derive(Debug)]
    pub struct IniFile {
        entries: Vec<Entry>,
    }

    /// Each entry is a section header, key=value pair, or comment
    #[derive(Debug)]
    pub enum Entry {
        Section(Section),
        Pair(Pair),
        Comment(Comment),
    }

    /// Section header: [name]
    #[derive(Debug)]
    pub struct Section {
        #[adze::leaf(text = "[")]
        _open: (),
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        name: String,
        #[adze::leaf(text = "]")]
        _close: (),
    }

    /// Key=value pair
    #[derive(Debug)]
    pub struct Pair {
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        key: String,
        #[adze::leaf(text = "=")]
        _eq: (),
        #[adze::leaf(pattern = r"[^\n\r]+", transform = |v| v.to_string())]
        value: String,
    }

    /// Comment line starting with # or ;
    #[derive(Debug)]
    pub struct Comment {
        #[adze::leaf(pattern = r"[#;][^\n\r]*", transform = |v| v.to_string())]
        text: String,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ini_section_and_pair() {
        insta::assert_debug_snapshot!(grammar::parse("[section]\nkey=value"));
    }

    #[test]
    fn ini_comment_section_pair() {
        insta::assert_debug_snapshot!(grammar::parse("# comment\n[s]\nk=v"));
    }

    #[test]
    fn ini_empty_file() {
        // An empty INI file is valid (empty entries list)
        insta::assert_debug_snapshot!(grammar::parse(""));
    }
}
