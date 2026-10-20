use std::{
    env,
    fs::File,
    io::{self, BufReader},
};

use fileformat::mdx::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: parse_mdx <file.mdx>")?;
    let file = File::open(&path)
        .map_err(|error| io::Error::new(error.kind(), format!("failed to open {path}: {error}")))?;
    let mut reader = BufReader::new(file);
    let model = Model::read(&mut reader).map_err(|error| {
        io::Error::new(error.kind(), format!("failed to parse {path}: {error}"))
    })?;

    println!("parsed {path}: {} nodes", model.nodes().count());
    Ok(())
}
