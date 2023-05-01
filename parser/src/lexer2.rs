//! Longest matching prefix

use std::rc::Rc;

use unicode_ident::{is_xid_continue, is_xid_start};

use crate::sym::{SymDB, SymData};


pub enum Token {
    Literal(String),
    Ident(String),
    Op(Rc<SymData>),
    PrthL,
    PrthR,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MatchState {
    Growable,
    Finished,
    Failure,
}

pub struct Lexer {
    s: String,
    buf_state: MatchState,
    tokens: Vec<Token>,

    lit_m: MatchState,
    lit_end: usize,
    lit_token: Option<Token>,

    idt_m: MatchState,
    idt_end: usize,
    idt_token: Option<Token>,

    kwd_m: MatchState,
    kwd_end: usize,
    kwd_token: Option<Token>,
    kwd_candidates: Vec<Rc<SymData>>,
    pub kwds: SymDB,

    ctn_m: MatchState,
    ctn_end: usize,
    ctn_token: Option<Token>,
}

enum PushResult {
    Datached(String),
    Failure,
    Growable,
}

impl Lexer {
    pub fn new(syms: SymDB) -> Self {
        Self {
            s: String::new(),
            buf_state: MatchState::Growable,
            tokens: Vec::new(),
            lit_m: MatchState::Growable,
            lit_end: 0,
            lit_token: None,
            idt_m: MatchState::Growable,
            idt_end: 0,
            idt_token: None,
            kwd_m: MatchState::Growable,
            kwd_end: 0,
            kwd_token: None,
            kwd_candidates: syms.iter().collect(),
            kwds: syms,
            ctn_m: MatchState::Growable,
            ctn_end: 0,
            ctn_token: None,
        }
    }
    pub fn clear(&mut self) {
        self.s.clear();
        self.buf_state = MatchState::Growable;
        self.lit_m = MatchState::Growable;
        self.lit_end = 0;
        self.lit_token = None;
        self.idt_m = MatchState::Growable;
        self.idt_end = 0;
        self.idt_token = None;
        self.kwd_m = MatchState::Growable;
        self.kwd_end = 0;
        self.kwd_token = None;
        self.kwd_candidates = self.kwds.iter().collect();
        self.ctn_m = MatchState::Growable;
        self.ctn_end = 0;
        self.ctn_token = None;
    }
    pub fn delimit(&mut self) -> MatchState {
        self.push_str(" ")
    }
    pub fn push(&mut self, c: char) -> MatchState {
        let mut buf = [0; 4];
        self.push_str(c.encode_utf8(&mut buf))
    }
    pub fn push_str(&mut self, s: &str) -> MatchState {
        let mut input = s.to_string();
        let mut chars = input.chars();
        'g: loop {
            while let Some(c) = chars.next() {
                match self.push_primitive(c) {
                    PushResult::Growable => {}
                    PushResult::Failure => {
                        self.s.extend(chars);
                        return MatchState::Failure;
                    }
                    PushResult::Datached(mut s) => {
                        s.extend(chars);
                        input = s;
                        chars = input.chars();
                        continue 'g;
                    }
                }
            }
            break;
        }
        MatchState::Growable
    }
    fn push_primitive(&mut self, c: char) -> PushResult {
        if self.buf_state == MatchState::Failure {
            self.s.push(c);
            return PushResult::Failure;
        }
        if c.is_whitespace() {
            if self.s.is_empty() {
                return PushResult::Growable;
            }
        }
        self.lit_push(c);
        self.idt_push(c);
        self.kwd_push(c);
        self.ctn_push(c);
        self.s.push(c);
        if self.lit_is_growable() || self.idt_is_growable()
        || self.kwd_is_growable() || self.ctn_is_growable() {
            return PushResult::Growable;
        } else {
            let (idx, end) =
                [self.lit_end, self.idt_end, self.kwd_end, self.ctn_end]
                .into_iter().enumerate().max_by_key(|&(_, end)| end).unwrap();
            if end == 0 {
                self.buf_state = MatchState::Failure;
                return PushResult::Failure;
            }
            let longest_token = match idx {
                0 => self.lit_token.take().unwrap(),
                1 => self.idt_token.take().unwrap(),
                2 => self.kwd_token.take().unwrap(),
                3 => self.ctn_token.take().unwrap(),
                _ => { unreachable!() },
            };
            self.tokens.push(longest_token);
            let unprocessed = self.s[end..].trim_start().to_string();
            self.clear();
            return PushResult::Datached(unprocessed);
        }
    }
    pub fn recv_token(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens.remove(0))
        }
    }
    pub fn recv_tokens(&mut self) -> Vec<Token> {
        std::mem::replace(&mut self.tokens, Vec::new())
    }
}

// Literal
impl Lexer {
    fn lit_push(&mut self, c: char) -> MatchState {
        if let MatchState::Growable = self.lit_m {
            if c.is_ascii_digit() {
            } else if self.s.is_empty() {
                self.lit_m = MatchState::Failure;
            } else {
                self.lit_m = MatchState::Finished;
                self.lit_end = self.s.len();
                self.lit_token = Some(Token::Literal(self.s.to_string()));
            }
        }
        self.lit_m
    }

    #[inline]
    fn lit_is_growable(&self) -> bool {
        self.lit_m == MatchState::Growable
    }
}

fn is_id_start(c: char) -> bool {
    is_xid_start(c) || c == '_' || c == '@'
}

// Identifier
impl Lexer {
    fn idt_push(&mut self, c: char) -> MatchState {
        if let MatchState::Growable = self.idt_m {
            if self.s.is_empty() {
                if !is_id_start(c) {
                    self.idt_m = MatchState::Failure;
                }
            } else {
                if !is_xid_continue(c) {
                    self.idt_m = MatchState::Finished;
                    self.idt_end = self.s.len();
                    self.idt_token = Some(Token::Ident(self.s.to_string()));
                }
            }
        }
        self.idt_m
    }

    #[inline]
    fn idt_is_growable(&self) -> bool {
        self.idt_m == MatchState::Growable
    }
}

// Keyword
impl Lexer {
    fn kwd_push(&mut self, c: char) -> MatchState {
        if MatchState::Growable != self.kwd_m {
            return self.kwd_m;
        }
        self.kwd_candidates.retain(|sym| {
            let kw_c = sym.name[self.s.len()..].chars().next().unwrap();
            if c == kw_c {
                let matched_bytes = self.s.len() + c.len_utf8();
                if matched_bytes != sym.name.len() {
                    return true;
                }
                self.kwd_end = matched_bytes;
                self.kwd_token = Some(Token::Op(Rc::clone(sym)));
            }
            false
        });
        if self.kwd_candidates.is_empty() {
            if self.kwd_token.is_some() {
                self.kwd_m = MatchState::Finished;
            } else {
                self.kwd_m = MatchState::Failure;
            }
        }
        self.kwd_m
    }

    #[inline]
    fn kwd_is_growable(&self) -> bool {
        self.kwd_m == MatchState::Growable
    }
}

// Container
impl Lexer {
    fn ctn_push(&mut self, c: char) -> MatchState {
        if MatchState::Growable != self.ctn_m {
            return self.ctn_m;
        }
        match c {
            '(' => {
                self.ctn_m = MatchState::Finished;
                self.ctn_end = '('.len_utf8();
                self.ctn_token = Some(Token::PrthL);
            }
            ')' => {
                self.ctn_m = MatchState::Finished;
                self.ctn_end = ')'.len_utf8();
                self.ctn_token = Some(Token::PrthR);
            }
            _ => {
                self.ctn_m = MatchState::Failure;
            }
        }
        self.ctn_m
    }

    #[inline]
    fn ctn_is_growable(&self) -> bool {
        self.ctn_m == MatchState::Growable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xid() {
        assert!(!is_xid_continue(' '));
        assert!(!is_xid_start('_'));
        assert!(is_xid_continue('_'));
        assert!(!is_xid_start('@'));
        assert!(!is_xid_start('$'));
        assert!(is_xid_start('あ'));
        assert!(is_xid_start('α'));
    }

    #[test]
    fn lexer() {
        let syms = SymDB::load("default.txt").unwrap();
        let mut lexer = Lexer::new(syms);
        let s = "(2+13)*x = alpha";
        lexer.push_str(s);
        lexer.delimit();
        let mut tokens = lexer.recv_tokens().into_iter();
        
        if let Some(Token::PrthL) = tokens.next() {
        } else { panic!() }
        if let Some(Token::Literal(s)) = tokens.next() {
            assert_eq!(s, "2");
        } else { panic!() }
        if let Some(Token::Op(s)) = tokens.next() {
            assert_eq!(s.name, "+");
        } else { panic!() }
        if let Some(Token::Literal(s)) = tokens.next() {
            assert_eq!(s, "13");
        } else { panic!() }
        if let Some(Token::PrthR) = tokens.next() {
        } else { panic!() }
        if let Some(Token::Op(s)) = tokens.next() {
            assert_eq!(s.name, "*");
        } else { panic!() }
        if let Some(Token::Ident(s)) = tokens.next() {
            assert_eq!(s, "x");
        } else { panic!() }
        if let Some(Token::Op(s)) = tokens.next() {
            assert_eq!(s.name, "=");
        } else { panic!() }
        if let Some(Token::Ident(s)) = tokens.next() {
            assert_eq!(s, "alpha");
        } else { panic!() }
    }
}
