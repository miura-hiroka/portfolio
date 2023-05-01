
use std::{fmt, rc::Rc};
use tree::v3::Tree;
use std::collections::BTreeMap;
use crate::lexer2::Token;
use crate::sym::{SymData, Assoc};


enum ProcItem {
    // Not done
    PrthL,
    PrthR,
    Sym(Rc<SymData>),

    // Done
    Done(Tree<String>),
}

impl ProcItem {
    fn is_sym(&self) -> bool {
        if let Self::Sym(_) = self { true } else { false }
    }
    fn get_sym(&self) -> Option<&Rc<SymData>> {
        if let &Self::Sym(ref info) = self { Some(info) }
        else { None }
    }
    fn unwrap_done(self) -> Tree<String> {
        if let Self::Done(tree) = self { tree }
        else { panic!() }
    }
}

impl fmt::Debug for ProcItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcItem::PrthL => {
                write!(f, "ParenL")
            }
            ProcItem::PrthR => {
                write!(f, "ParenR")
            }
            ProcItem::Sym(op) => {
                write!(f, "{}", op.name)
            }
            ProcItem::Done(t) => {
                write!(f, "Tree{{{}}}", t)
            }
        }
    }
}

impl From<Token> for ProcItem {
    fn from(token: Token) -> Self {
        match token {
            Token::PrthL => {
                ProcItem::PrthL
            }
            Token::PrthR => {
                ProcItem::PrthR
            }
            Token::Literal(s) => {
                ProcItem::Done(Tree::new(s))
            }
            Token::Ident(s) => {
                ProcItem::Done(Tree::new(s))
            }
            Token::Op(sym) => {
                ProcItem::Sym(sym)
            }
        }
    }
}

fn sort_by_prec(tokens: &BTreeMap<usize, ProcItem>, start: usize, end: usize) -> Vec<usize> {
    use std::cmp::Ordering;
    let mut ascending: Vec<usize> = tokens.range(start..end)
        .filter_map(|(&key, token)|
            if token.is_sym() { Some(key) } else { None }
        )
        .collect();
    let compare =
    |idx0: &usize, idx1: &usize| -> Ordering {
        let sym0 = tokens.get(idx0).unwrap().get_sym().unwrap();
        let sym1 = tokens.get(idx1).unwrap().get_sym().unwrap();
        let prec0 = sym0.prec;
        let prec1 = sym1.prec;
        if prec0 < prec1 {
            Ordering::Less
        } else if prec0 > prec1 {
            Ordering::Greater
        } else if sym0.assoc == Assoc::L {
            idx1.cmp(idx0)
        } else {
            idx0.cmp(idx1)
        }
    };
    ascending.sort_unstable_by(compare);
    ascending
}

// no parentheses
fn gen_ast_no_paren(tokens: &mut BTreeMap<usize, ProcItem>, start: usize, end: usize) -> Result<(), &'static str> {
    use std::ops::Bound::{Excluded, Included};
    let prec_indices = sort_by_prec(tokens, start, end);
    for idx in prec_indices.into_iter().rev() {
        let sym = tokens.get(&idx).unwrap().get_sym().unwrap();
        let sym = &*Rc::clone(sym);
        
        let front = sym.layout.front;
        let mut args = vec![0; sym.arity()];
        let mut args_fr = tokens.range(start..idx).rev()
            .map(|(&i, _)| i);
        for i in (0..front).rev() {
            args[i] = args_fr.next().ok_or_else(|| "insufficient operands")?;
        }
        let mut args_bk = tokens.range((Excluded(idx), Included(end)))
            .map(|(&i, _)| i);
        for i in front..front+sym.layout.back {
            args[i] = args_bk.next().ok_or_else(|| "insufficient operands")?;
        }
        let mut op_tree = Tree::new(sym.name.clone());
        let root_id = op_tree.root_id();
        for i in args {
            let arg = tokens.remove(&i).unwrap();
            let ProcItem::Done(operand) = arg else { return Err("syntax error 2") };
            op_tree.push_tree(root_id, operand);
        }
        *tokens.get_mut(&idx).unwrap() = ProcItem::Done(op_tree);
    }
    if tokens.range(start..end).count() == 1 {
        Ok(())
    } else {
        Err("syntax error 3")
    }
}

pub struct AstGen {
    tokens: BTreeMap<usize, ProcItem>,
    num_recv: usize,
    paren_l: Vec<usize>,
}

impl AstGen {
    pub fn new() -> Self {
        Self { tokens: BTreeMap::new(), num_recv: 0, paren_l: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.num_recv = 0;
        self.paren_l.clear();
    }

    pub fn recv(&mut self, token: Token) -> Result<(), &'static str> {
        let i = self.num_recv;
        self.num_recv += 1;
        match token {
            Token::PrthL => {
                self.paren_l.push(i);
            }
            Token::PrthR => {
                let Some(paren_l) = self.paren_l.pop()
                    else { return Err("missing ParenL"); };
                gen_ast_no_paren(&mut self.tokens, paren_l, i)?;
            }
            Token::Literal(s) => {
                self.tokens.insert(i, ProcItem::Done(Tree::new(s)));
            }
            Token::Ident(s) => {
                self.tokens.insert(i, ProcItem::Done(Tree::new(s)));
            }
            Token::Op(op) => {
                self.tokens.insert(i, ProcItem::Sym(op));
            }
        }
        Ok(())
    }

    pub fn finish(&mut self) -> Result<Tree<String>, &'static str> {
        gen_ast_no_paren(&mut self.tokens, 0, self.num_recv)?;
        if self.tokens.range(..).count() == 1 {
            Ok(self.tokens.pop_first().unwrap().1.unwrap_done())
        } else {
            Err("syntax error 4")
        }
    }

    pub fn recv_all<I>(&mut self, tokens: I) -> Result<Tree<String>, &'static str>
    where I: Iterator<Item = Token> {
        for token in tokens {
            self.recv(token)?;
        }
        self.finish()
    }
}

impl Default for AstGen {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sym::SymDB;
    use crate::lexer2::Lexer;

    #[test]
    fn it_works() {
        let syms = SymDB::load("default.txt").unwrap();
        let mut lexer = Lexer::new(syms);
        let mut parser = AstGen::new();

        lexer.push_str("");
        lexer.delimit();
        let tokens = lexer.recv_tokens();
        let result = parser.recv_all(tokens.into_iter());
        assert!(result.is_err());

        parser.clear();
        lexer.push_str("-a + (b * c)");
        lexer.delimit();
        let tokens = lexer.recv_tokens();
        let ast = parser.recv_all(tokens.into_iter()).unwrap();
        assert_eq!(format!("{}", ast), "+(-(a) *(b c))");
    }
}
