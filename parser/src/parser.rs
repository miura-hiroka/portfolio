
use std::collections::HashMap;

use tree::v3::{Tree, Subtree};
use crate::sym::SymDB;
use crate::lexer2::{Lexer, MatchState};
use crate::ast_btree::AstGen;

pub struct Parser {
    lexer: Lexer,
    parser: AstGen,
}

impl Parser {
    pub fn new(file_name: &str) -> Result<Self, String> {
        Ok(Self { lexer: Lexer::new(SymDB::load(file_name)?), parser: AstGen::new() })
    }
    pub fn clear(&mut self) {
        self.lexer.clear();
        self.parser.clear();
    }
    pub fn parse(&mut self, s: &str) -> Result<Tree<String>, String> {
        if MatchState::Failure == self.lexer.push_str(s) {
            self.clear();
            return Err("push_str: MatchState::Failure".to_string());
        }
        if MatchState::Failure == self.lexer.delimit() {
            self.clear();
            return Err("delimit: MatchState::Failure".to_string());
        }
        let tokens = self.lexer.recv_tokens();
        let result = self.parser.recv_all(tokens.into_iter());
        self.clear();
        result.map_err(|err| err.to_string())
    }
    
    pub fn symdb(&self) -> &SymDB {
        &self.lexer.kwds
    }

    pub fn pattern_match(&mut self, ast: Subtree<String>, pat: &str)
    -> Result<HashMap<String, Tree<String>>, String> {
        let pat = self.parse(pat)?;
        let mut map = HashMap::new();
        if pattern_match_sub(ast, pat.subtree_root(), &mut map) {
            Ok(map)
        } else {
            Err("`ast` does not match `pat`".to_string())
        }
    }

}


pub fn pattern_match_tree(ast: Subtree<String>, pat: Subtree<String>)
-> Result<HashMap<String, Tree<String>>, ()> {
    let mut map = HashMap::new();
    if pattern_match_sub(ast, pat, &mut map) {
        Ok(map)
    } else {
        Err(())
    }
}

fn pattern_match_sub(ast: Subtree<String>, pat: Subtree<String>, map: &mut HashMap<String, Tree<String>>) -> bool {
    let pat_value = pat.get_root();
    if pat_value == "_" {
        return true;
    }
    if let Some(var) = pat_value.strip_prefix('@') {
        if let Some(tree) = map.get(var) {
            return tree.subtree_root() == ast;
        } else {
            map.insert(var.to_string(), ast.to_owned());
            return true;
        }
    }
    if ast.get_root() != pat_value {
        return false;
    }
    if ast.children_len() != pat.children_len() {
        return false;
    }
    std::iter::zip(ast.children_root(), pat.children_root())
        .all(|(t0, t1)| pattern_match_sub(t0, t1, map))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut pe = Parser::new("default.txt").unwrap();

        let result = pe.parse("");
        assert!(result.is_err());

        let result = pe.parse("a + b + ");
        assert!(result.is_err());

        let ast = pe.parse("a").unwrap();
        let simplified = pe.symdb().ast_to_string_minimal(&ast);
        assert_eq!(simplified, "a");

        let ast = pe.parse("(a)").unwrap();
        let simplified = pe.symdb().ast_to_string_minimal(&ast);
        assert_eq!(simplified, "a");

        let ast = pe.parse("!!(--a = a)").unwrap();
        let simplified = pe.symdb().ast_to_string_minimal(&ast);
        assert_eq!(simplified, "! ! - - a = a");

        let ast = pe.parse("(a^2 + 3) * (4*b) = 4*a^2*b + 12*b").unwrap();
        let pat = pe.parse("(@alpha + 3) * _ = 4*@alpha*b + @beta*b").unwrap();
        let map = pattern_match_tree(ast.subtree_root(), pat.subtree_root()).unwrap();
        assert_eq!(map.get("alpha"), Some(&pe.parse("a^2").unwrap()));
        assert_eq!(map.get("beta"), Some(&pe.parse("12").unwrap()));
        assert_eq!(map.len(), 2);
    }
}
