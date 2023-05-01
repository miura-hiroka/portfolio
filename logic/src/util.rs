
pub struct SplitWhitespace<'a> {
    s: &'a str,
}

impl<'a> From<&'a str> for SplitWhitespace<'a> {
    fn from(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> SplitWhitespace<'a> {
    pub fn remainder(&self) -> &str {
        self.s
    }
}

impl<'a> Iterator for SplitWhitespace<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let trimmed = self.s.trim_start();
        if trimmed.is_empty() {
            self.s = "";
            return None;
        }
        let Some((next, rem)) = trimmed.split_once(char::is_whitespace) else {
            self.s = "";
            return Some(trimmed);
        };
        self.s = rem;
        Some(next)
    }
}
