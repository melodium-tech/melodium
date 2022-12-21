
use std::io::{Read, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use brotli::Decompressor;
use tar::Archive;

#[derive(PartialEq, Eq, Debug)]
pub struct Jeu {

    files: HashMap<PathBuf, Vec<u8>>,
}

impl Jeu {

    pub fn new<R: Read>(mut reader: R) -> Result<Self> {

        let mut encountered_lf = 0;

        let mut byte: u8 = 0;
        while encountered_lf < 2 {

            reader.read_exact(std::slice::from_mut(&mut byte))?;
            if byte == 0x0A {
                encountered_lf += 1;
            }
        }

        let mut archive = Archive::new(Decompressor::new(reader, 4096));

        let mut files = HashMap::new();

        for entry in archive.entries()? {
            match entry {
                Ok(mut entry) => {

                    let path = entry.path()?.to_path_buf();
                    if path.ends_with(".mel") {

                        let mut data = Vec::new();
                        entry.read_to_end(&mut data)?;

                        files.insert(path, data);
                    }
                }
                Err(e) => return Err(e)
            }
        }

        Ok(Self {
            files,
        })
    }

    pub fn entries(&self) -> Vec<&Path> {
        self.files.keys().map(|k| k.as_path()).collect()
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Vec<u8>> {
        self.files.get(&path.as_ref().to_path_buf())
    }

}