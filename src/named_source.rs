#[derive(Clone, PartialEq)]
pub struct NamedSource {
    pub(crate) name: String,
    pub(crate) bytes: String,
    pub(crate) chars: Vec<char>,
}

impl NamedSource {
    pub fn new(name: String, bytes: String) -> NamedSource {
        let chars = bytes.chars().collect();
        NamedSource { name, bytes, chars }
    }
}

impl std::fmt::Debug for NamedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}