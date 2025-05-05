use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
};

use anyhow::Result;

pub(crate) fn read_lines<P>(path: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(BufReader::new(file).lines())
}
