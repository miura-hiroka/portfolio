use std::{io, env};

fn main() -> Result<(), &'static str> {
    
    let mut arg_iter = env::args_os();

    if arg_iter.next().is_none() {
        panic!();
    }

    let input = io::stdin();
    let mut buf = String::new();

    loop {
        if let Err(err) = input.read_line(&mut buf) {
            println!("{}", err);
            return Err("input error");
        }
        if buf == "exit" {
            break;
        }
        buf.clear();
    }
    println!("Exiting...");
    Ok(())
}
