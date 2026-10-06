use std::{env, fs::File, io::BufReader};

use model::mdx::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: parse_mdx <file.mdx>")?;
    let mut reader = BufReader::new(File::open(path)?);
    let model = Model::read(&mut reader)?;
    println!("{model:#?}");
    Ok(())
}
