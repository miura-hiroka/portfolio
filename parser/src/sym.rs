use std::str::FromStr;
use std::{rc::Rc, io::BufRead};
use std::fs::File;
use std::io::BufReader;
use tree::v3::{Tree, Subtree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc {
    L,
    R,
}

impl FromStr for Assoc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            _ => Err(format!("cannot parse \"{}\" as Assoc", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub front: usize,
    pub back: usize,
}

impl Layout {
    #[inline]
    pub fn new(front: usize, back: usize) -> Self {
        Self { front, back }
    }
    pub fn arity(&self) -> usize {
        self.front + self.back
    }
}

#[derive(Debug, Clone)]
pub struct SymData {
    pub name: String,
    pub layout: Layout,
    pub prec: i32,
    pub assoc: Assoc,
}

impl SymData {
    #[inline]
    pub fn arity(&self) -> usize {
        self.layout.arity()
    }
    #[inline]
    pub fn is_nullary(&self) -> bool {
        self.arity() == 0
    }

    pub fn new(name: &str, front: usize, back: usize, prec: i32, assoc: Assoc) -> Self {
        Self {
            name: name.to_string(),
            layout: Layout { front, back },
            prec,
            assoc,
        }
    }

    pub fn new_ident(ident: &str) -> Self {
        Self {
            name: ident.to_string(),
            layout: Layout { front: 0, back: 0 },
            prec: SymDB::NULLARY_PREC,
            assoc: Assoc::L,
        }
    }
}

pub struct SymDB {
    symbols: Vec<Rc<SymData>>,
}

impl SymDB {
    const NULLARY_PREC: i32 = i32::MAX;

    pub fn new() -> Self {
        Self { symbols: Vec::new() }
    }

    pub fn push(&mut self, sym: SymData) {
        self.symbols.push(Rc::new(sym));
    }

    pub fn push_nullary(&mut self, s: &str) {
        let info = SymData {
            name: s.to_string(),
            prec: Self::NULLARY_PREC,
            assoc: Assoc::L,
            layout: Layout { front: 0, back: 0 },
        };
        self.push(info);
    }
    pub fn get(&self, s: &str) -> Rc<SymData> {
        self.symbols.iter()
            .rfind(|&info| info.name == s)
            .map_or_else(
                || Rc::new(SymData::new_ident(s)),
                |info| Rc::clone(info)
            )
    }
    pub fn iter_names(&self) -> IterNames {
        IterNames { next_idx: 0, syms: self.symbols.as_slice() }
    }
    pub fn iter(&self) -> Iter {
        Iter { syms: self.symbols.as_slice(), next_idx: 0 }
    }
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
    pub fn is_variable(&self, s: &str) -> bool {
        self.symbols.iter()
            .all(|info| info.name != s)
    }

    pub fn load(file_name: &str) -> Result<Self, String> {
        let reader = BufReader::new(File::open(file_name).map_err(|err| err.to_string())?);
        let mut symdb = SymDB::new();
        let mut prec = i32::MAX;
        for line in reader.lines() {
            let line = line.map_err(|err| err.to_string())?;
            let line = line.split('#').next().unwrap_or(&line);
            let mut tokens = line.split_whitespace();
            let Some(name) = tokens.next() else { continue; };
            let front = tokens.next().ok_or_else(|| "syntax error".to_string())?;
            let front = front.parse::<usize>().map_err(|err| err.to_string())?;
            let back = tokens.next().ok_or_else(|| "syntax error".to_string())?;
            let back = back.parse::<usize>().map_err(|err| err.to_string())?;
            let assoc = if let Some(assoc) = tokens.next() {
                prec -= 1;
                assoc.parse()?
            } else {
                if let Some(prev) = symdb.symbols.last() {
                    prev.assoc
                } else {
                    return Err("cannot omit the associativity of the first entry".to_string());
                }
            };
            let sym = SymData::new(name, front, back, prec, assoc);
            symdb.push(sym);
        }
        Ok(symdb)
    }

    // marks each node that need parentheses as true.
    fn ast_to_ast_prth(&self, sub: &Subtree<'_, String>) -> Tree<(String, bool)> {
        let name = sub.get_root();
        let mut clone = Tree::new((name.clone(), false));
        let sym_data = self.get(name);
        let children = sub.children_root();
        for (i, child) in children.enumerate() {
            let child_sym_data = self.get(child.get_root());
            let mut child_tree = self.ast_to_ast_prth(&child);
            child_tree.get_mut_root().unwrap().1 = if child_sym_data.prec < sym_data.prec {
                true
            } else if child_sym_data.prec > sym_data.prec {
                false
            } else {
                if i < sym_data.layout.front {
                    sym_data.assoc == Assoc::R
                } else {
                    sym_data.assoc == Assoc::L
                }
            };
            clone.push_tree(clone.root_id(), child_tree);
        }
        clone
    }

    
    // Converts `Tree` with parentheses infomation to `String`.
    fn ast_prth_to_string(&self, subtree: Subtree<(String, bool)>) -> String {
        let mut s = String::new();
        let &(ref name, need_prth) = subtree.get_root();
        let sym_data = self.get(name);
        let arity = sym_data.arity();
        if arity == 0 {
            return name.clone();
        }
        for (i, child) in subtree.children_root().enumerate() {
            if i == sym_data.layout.front {
                s += name;
                if sym_data.layout.back == 0 {
                    break;
                }
                s.push(' ');
            }
            let child_str = self.ast_prth_to_string(child);
            s += child_str.as_str();
            if i + 1 == arity {
                break;
            }
            s.push(' ');
        }
        if need_prth { format!("({})", s) } else { s }
    }

    /// Converts `Tree` to `String` with as few parentheses as possible.
    pub fn ast_to_string_minimal(&self, tree: &Tree<String>) -> String {
        let tree_prth = self.ast_to_ast_prth(&tree.subtree_root());
        self.ast_prth_to_string(tree_prth.subtree_root())
    }

}

pub struct IterNames<'a> {
    syms: &'a [Rc<SymData>],
    next_idx: usize,
}

impl<'a> Iterator for IterNames<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.syms.get(self.next_idx).map(|sym_data| {
            self.next_idx += 1;
            sym_data.name.as_str()
        })
    }
}

pub struct Iter<'a> {
    syms: &'a [Rc<SymData>],
    next_idx: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Rc<SymData>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.syms.len() {
            let next = Rc::clone(&self.syms[self.next_idx]);
            self.next_idx += 1;
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sym_db() {
        let sym_db = SymDB::load("default.txt").unwrap();
        assert_eq!(sym_db.symbols.len(), 16);
        let mul = sym_db.get("*");
        assert_eq!(mul.assoc, Assoc::L);
        assert_eq!(mul.layout.front, 1);
        assert_eq!(mul.layout.back, 1);        
        let div = sym_db.get("/");
        assert_eq!(div.assoc, Assoc::L);
        assert_eq!(div.layout.front, 1);
        assert_eq!(div.layout.back, 1);
        assert_eq!(mul.prec, div.prec);
        let add = sym_db.get("+");
        assert!(add.prec < mul.prec);
    }
}
