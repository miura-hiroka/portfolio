use std::{io, env};
use logic::proposition::System;

fn main() -> Result<(), &'static str> {
    
    let mut arg_iter = env::args_os();

    assert!(arg_iter.next().is_some());

    let input = io::stdin();
    let mut buf = String::new();
    let mut sys = System::new("ops.txt", "prop_axioms.txt").unwrap();

    loop {
        if let Err(err) = input.read_line(&mut buf) {
            println!("{}", err);
            return Err("input error");
        }
        println!("{} bytes read", buf.len());
        let trimmed = buf.trim();
        if trimmed == "exit" {
            break;
        }
        match sys.command(trimmed) {
            Err(err) => {
                println!("Error: {:?}", err);
            }
            Ok(_) => {}
        }
        buf.clear();
    }
    println!("Exiting...");
    Ok(())
}
