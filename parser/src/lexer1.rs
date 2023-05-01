//! split strings by char class boundaries

use std::str::Chars;



#[inline]
pub fn first_char(s: &str) -> Option<char> {
    s.chars().next()
}

#[inline]
pub fn cond_ws(c: char) -> bool {
    c.is_whitespace()
}

#[inline]
fn cond_bracket_l(c: char) -> bool {
    c == '(' || c == '{' || c == '['
}

#[inline]
fn cond_bracket_r(c: char) -> bool {
    c == ')' || c == '}' || c == ']'
}

#[inline]
fn cond_normal(c: char) -> bool {
    unicode_ident::is_xid_continue(c)
}

#[derive(PartialEq, Eq)]
enum CharClass {
    Whitespace,
    Punct,
    Ident,
    BracketL,
    BracketR,
}

fn class_of(c: char) -> CharClass {
    if cond_ws(c) { CharClass::Whitespace }
    else if cond_bracket_l(c) { CharClass::BracketL }
    else if cond_bracket_r(c) { CharClass::BracketR }
    else if cond_normal(c) { CharClass::Ident }
    else { CharClass::Punct }
}

#[inline]
pub fn is_ws(c: char) -> bool {
    class_of(c) == CharClass::Whitespace
}

#[inline]
pub fn is_not_ws(c: char) -> bool {
    class_of(c) != CharClass::Whitespace
}

#[inline]
pub fn is_bracket_l(c: char) -> bool {
    class_of(c) == CharClass::BracketL
}

#[inline]
pub fn is_bracket_r(c: char) -> bool {
    class_of(c) == CharClass::BracketR
}

pub fn is_bracket(c: char) -> bool {
    is_bracket_l(c) || is_bracket_r(c)
}

#[inline]
pub fn is_normal(c: char) -> bool {
    class_of(c) == CharClass::Ident
}

#[inline]
pub fn is_punctuation(c: char) -> bool {
    class_of(c) == CharClass::Punct
}

// if c1 or c2 is a not bracket, return false
pub fn same_bracket(c1: char, c2: char) -> bool {
    let pairs = [('(', ')'), ('{', '}'), ('[', ']'),];
    let bracket_id =
        |c| pairs.iter().position(|p| p.0 == c || p.1 == c);
    let Some(i1) = bracket_id(c1) else { return false };
    let Some(i2) = bracket_id(c2) else { return false };
    i1 == i2
}

#[derive(PartialEq, Eq)]
pub enum TokenClass {
    Punct,
    Ident,
    BracketL,
    BracketR,
}

pub fn token_class_of(s: &str) -> Option<TokenClass> {
    let c0 = s.chars().next()?;
    match class_of(c0) {
        CharClass::Whitespace => None,
        CharClass::Punct => Some(TokenClass::Punct),
        CharClass::Ident => Some(TokenClass::Ident),
        CharClass::BracketL => Some(TokenClass::BracketL),
        CharClass::BracketR => Some(TokenClass::BracketR),
    }
}

pub struct IterToken<'a> {
    s: &'a str,
}

impl<'a> From<&'a str> for IterToken<'a> {
    fn from(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> Iterator for IterToken<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.s = self.s.trim_start();
        let c0 = self.s.chars().next()?;
        let c0_cls = class_of(c0);
        let tk_byte = match c0_cls {
            CharClass::Whitespace => panic!(),
            CharClass::BracketL => 1,
            CharClass::BracketR => 1,
            _ => {
                self.s.find(|c| class_of(c) != c0_cls)
                    .unwrap_or_else(|| self.s.len())
            }
        };
        let tk = &self.s[..tk_byte];
        self.s = &self.s[tk_byte..];
        Some(tk)
    }
}

struct IterTokenClass<'a> {
    s: &'a str,
}

impl<'a> From<&'a str> for IterTokenClass<'a> {
    fn from(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> Iterator for IterTokenClass<'a> {
    type Item = (&'a str, TokenClass);
    fn next(&mut self) -> Option<Self::Item> {
        self.s = self.s.trim_start();
        let c0 = self.s.chars().next()?;
        let c0_cls = class_of(c0);
        let get_tk_byte =
        ||  self.s.find(|c| class_of(c) != c0_cls)
            .unwrap_or_else(|| self.s.len());
        let (tk_byte, tk_cls) = match c0_cls {
            CharClass::Whitespace => panic!(),
            CharClass::BracketL => (1, TokenClass::BracketL),
            CharClass::BracketR => (1, TokenClass::BracketR),
            CharClass::Ident => (get_tk_byte(), TokenClass::Ident),
            CharClass::Punct => (get_tk_byte(), TokenClass::Punct),
        };
        let tk = &self.s[..tk_byte];
        self.s = &self.s[tk_byte..];
        Some((tk, tk_cls))
    }
}

struct BetweenChars<'a> {
    it: Chars<'a>,
    prev: char,
}

impl<'a> From<&'a str> for BetweenChars<'a> {
    fn from(s: &'a str) -> Self {
        let mut it = s.chars();
        if let Some(prev) = it.next() {
            BetweenChars { it, prev }
        } else {
            BetweenChars { it, prev: ' ' }
        }
    }
}

impl<'a> Iterator for BetweenChars<'a> {
    type Item = (char, char);
    fn next(&mut self) -> Option<Self::Item> {
        let c0 = self.prev;
        self.prev = self.it.next()?;
        Some((c0, self.prev))
    }
}

/*
a a
a #
a (
a)
# a
# #
# (
#)
(a
(#
((
()
) a
) #
) (
))
*/
pub fn cleanup(s: &str) -> String {
    let mut res = String::new();
    let mut it_tk = IterTokenClass::from(s);
    let Some((mut prev, mut prev_cls)) = it_tk.next() else { return res };
    res.push_str(prev);
    for (tk, tk_cls) in it_tk {
        let need_ws = match prev_cls {
            TokenClass::Ident | TokenClass::Punct => {
                match tk_cls {
                    TokenClass::Ident | TokenClass::Punct => { true }
                    TokenClass::BracketL => { true }
                    TokenClass::BracketR => { false }
                }
            }
            TokenClass::BracketL => {
                match tk_cls {
                    TokenClass::Ident | TokenClass::Punct => { false }
                    TokenClass::BracketL => { false }
                    TokenClass::BracketR => {
                        !same_bracket(first_char(prev).unwrap(), first_char(tk).unwrap())
                    }
                }
            }
            TokenClass::BracketR => {
                match tk_cls {
                    TokenClass::Ident | TokenClass::Punct => { true }
                    TokenClass::BracketL => { true }
                    TokenClass::BracketR => { false }
                }
            }
        };
        if need_ws { res.push(' '); }
        res.push_str(tk);
        prev = tk;
        prev_cls = tk_cls;
    }
    res
}


#[cfg(test)]
mod tests {
    use crate::lexer1::{is_normal, is_not_ws, cleanup};

    #[test]
    fn test() {
        assert!(!is_normal(' '));
        assert!(!unicode_ident::is_xid_start('_'));
        assert!(unicode_ident::is_xid_continue('_'));
        assert!(unicode_ident::is_xid_continue('あ'));
        assert!(!unicode_ident::is_xid_continue('、'));
        for c in '\u{0}'..='\u{7F}' {
            assert!(!is_normal(c) || is_not_ws(c));
        }
        let s = cleanup("");
        assert_eq!(s, "");
        let s = cleanup("\n(a+  b )*c\n");
        assert_eq!(s, "(a + b) * c");
        let s = cleanup("(){}[](}(]{){][)[}");
        assert_eq!(s, "() {} [] ( } ( ] { ) { ] [ ) [ }");

    }
}
