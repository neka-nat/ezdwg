use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::core::result::Result;

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut file = File::open(path.as_ref())?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub fn read_version_tag(path: impl AsRef<Path>) -> Result<[u8; 6]> {
    let mut file = File::open(path.as_ref())?;
    let mut tag = [0u8; 6];
    file.read_exact(&mut tag)?;
    Ok(tag)
}

pub fn read_header(path: impl AsRef<Path>, bytes: usize) -> Result<Vec<u8>> {
    let mut file = File::open(path.as_ref())?;
    let mut buf = vec![0u8; bytes];
    let _ = file.read(&mut buf)?;
    Ok(buf)
}

pub fn file_size(path: impl AsRef<Path>) -> Result<u64> {
    let mut file = File::open(path.as_ref())?;
    let size = file.seek(SeekFrom::End(0))?;
    Ok(size)
}
