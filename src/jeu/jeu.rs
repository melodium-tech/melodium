
use std::io::{Read, Result, Seek, SeekFrom};
use brotli::Decompressor;
use tar::Archive;

pub struct Jeu<R: AsMut<dyn Read> + Read + Seek + Sized> {

    archive: Archive<Decompressor<R>>,
}

impl<R: AsMut<dyn Read> + Read + Seek + Sized> Jeu<R> {

    pub fn new(mut reader: R) -> Result<Self> {

        let mut ignore_bytes = 0;
        let mut encountered_lf = 0;

        for byte in reader.as_mut().bytes() {
            match byte {
                Ok(byte) => {
                    ignore_bytes += 1;
                    if byte == 0x0A {
            
                        encountered_lf += 1;

                        if encountered_lf == 2 {
                            break;
                        }
                    }
                }
                Err(e) => {
                    return Err(e)
                }
            }
        }

        reader.seek(SeekFrom::Start(ignore_bytes))?;

        let archive = Archive::new(Decompressor::new(reader, 4096));

        Ok(Self {
            archive,
        })
    }

    pub fn get_all_entries(&self) {
        
    }


}