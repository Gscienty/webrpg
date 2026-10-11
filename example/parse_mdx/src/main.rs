use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use renderer::model::mdx::ModelChunk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: parse_mdx <file.mdx>")?;
    let mut reader = BufReader::new(File::open(path)?);

    let mut magic = [0; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"MDLX" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid model tag").into());
    }

    let mut chunks = 0;
    while !reader.fill_buf()?.is_empty() {
        ModelChunk::read(&mut reader)?;
        chunks += 1;
    }

    println!("parsed {chunks} chunks");
    Ok(())
}
