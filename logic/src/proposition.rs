use tree::v3::{Tree, Subtree};
use parser::parser::Parser;
use crate::util::SplitWhitespace;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::io::{Error, ErrorKind};

pub struct System {
    proofs: Vec<Tree<String>>,
    parser: Parser,
}

impl System {

    pub fn new(op_file: &str, axiom_file: &str) -> Result<Self, String> {
        let mut parser = Parser::new(op_file)?;
        let reader_ax = BufReader::new(File::open(axiom_file).map_err(|err| err.to_string())?);
        let mut proofs = Vec::new();
        for line in reader_ax.lines() {
            let line = line.map_err(|err| err.to_string())?;
            if line.is_empty() {
                continue;
            }
            proofs.push(parser.parse(&line)?);
        }
        Ok(Self { proofs, parser })
    }

    pub fn inst(&mut self, id: usize, var: &str, replace: Subtree<'_, String>) -> Result<usize, String> {
        let Some(proof) = self.proofs.get(id) else {
            return Err(format!("there is no proof with ID {}", id));
        };
        let mut new_proof = proof.clone();
        new_proof.replace(Tree::new(var.to_string()).subtree_root(), replace);
        let id = self.proofs.len();
        self.proofs.push(new_proof);
        Ok(id)
    }

    pub fn mp(&mut self, id_antec: usize, id_imply: usize) -> Result<usize, String> {
        let Some(proof_antec) = self.proofs.get(id_antec) else {
            return Err(format!("there is no proof with ID {}", id_antec));
        };
        let Some(proof_imply) = self.proofs.get(id_imply) else {
            return Err(format!("there is no proof with ID {}", id_imply));
        };
        if proof_imply.get_root().unwrap() != "->" {
            return Err("a mismatched form '[a] -> [b]'".to_string());
        }
        let mut operands = proof_imply.children_root();
        let antecedent = proof_imply.subtree(operands.next().unwrap());
        if proof_antec.subtree_root() != antecedent {
            return Err("a mismatched pattern '[a], [a] -> [b]'".to_string());
        }
        let consequent = proof_imply.clone_sub(operands.next().unwrap());
        let id = self.proofs.len();
        self.proofs.push(consequent);
        Ok(id)
    }

    pub fn print_proof(&self, id: usize) {
        let symdb = self.parser.symdb();
        let Some(proof) = self.proofs.get(id) else {
            return println!("an invalid proof ID: {}", id);
        };
        println!("{}: {}", id, symdb.ast_to_string_minimal(proof));
    }

    pub fn print_proofs(&self) {
        let symdb = self.parser.symdb();
        for (id, proof) in self.proofs.iter().enumerate() {
            println!("{}: {}", id, symdb.ast_to_string_minimal(proof));
        }
    }

    pub fn save(&self, file_name: &str) -> std::io::Result<()> {
        let mut buffer = BufWriter::new(File::create(file_name)?);
        let symdb = self.parser.symdb();
        for proof in self.proofs.iter() {
            let proof_str = symdb.ast_to_string_minimal(proof) + "\n";
            buffer.write_all(proof_str.as_bytes())?;
        }
        buffer.flush()?;
        Ok(())
    }

    pub fn load(&mut self, file_name: &str) -> std::io::Result<()> {
        let buffer = BufReader::new(File::open(file_name)?);
        self.proofs.clear();
        for line in buffer.lines() {
            let proof = self.parser.parse(&line?)
                .map_err(|err| Error::new(ErrorKind::Other, err))?;
            self.proofs.push(proof);
        }
        Ok(())
    }

    pub fn command(&mut self, s: &str) -> Result<(), String> {
        let mut args = SplitWhitespace::from(s);
        let Some(cmd) = args.next() else { return Ok(()); };
        match cmd {
            "show" => {
                self.print_proofs();
            }
            "save" => {
                let file_name = args.next().unwrap_or("default.txt");
                if let Err(err) = self.save(file_name) {
                    return Err(err.to_string());
                }
            }
            "load" => {
                let file_name = args.next().unwrap_or("default.txt");
                if let Err(err) = self.load(file_name) {
                    return Err(err.to_string());
                }
            }
            "mp" => {
                let Some(ant_id) = args.next() else { return Err("E1".to_string()); };
                let Some(imp_id) = args.next() else { return Err("E2".to_string()); };
                let ant_id: usize = ant_id.parse().map_err(|_| "E3".to_string())?;
                let imp_id: usize = imp_id.parse().map_err(|_| "E4".to_string())?;
                let new_id = self.mp(ant_id, imp_id)?;
                self.print_proof(new_id);
            }
            "inst" => {
                let Some(id) = args.next() else { return Err("".to_string()); };
                let id: usize = id.parse().map_err(|_| "".to_string())?;
                let Some(var) = args.next() else { return Err("".to_string()); };
                let rem = args.remainder();
                let replace = self.parser.parse(rem).map_err(|_| "".to_string())?;
                let new_id = self.inst(id, var, replace.subtree_root())?;
                self.print_proof(new_id);
            }
            other => { return Err(format!("unknown command: {}", other)); }
        }
        Ok(())
    }

}
