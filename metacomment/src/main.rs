use std::env;

mod comment_of_c;
mod comment_extractor;
use comment_extractor::*;

fn main() -> std::io::Result<()> {
    let mut args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <c code file>", args[0]);
        return Ok(());
    }
    let infile = args.remove(1);
    let infile = std::fs::File::open(infile)?;
    let ce = CommentExtractor::new(infile);
    for com in ce {
        println!("{}", com);
    }

    Ok(())
}
