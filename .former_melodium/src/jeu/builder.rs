
use std::io::{Write, Result};
use std::path::Path;
use brotli::enc::writer::CompressorWriter;
use tar::{Builder as TarBuilder, Header, HeaderMode};
pub struct Builder<W: Write> {

    archive: TarBuilder<CompressorWriter<W>>
}

impl<W: Write> Builder<W> {

    pub fn new(mut obj: W) -> Result<Self> {

        obj.write_all(b"#!/usr/bin/env melodium\nVERSION=0\n")?;

        let compressor = CompressorWriter::new(obj, 4096, 10, 4096);

        let mut archive = TarBuilder::new(compressor);
        archive.mode(HeaderMode::Deterministic);

        Ok(Self {
            archive
        })
    }

    pub fn append<P: AsRef<Path>>(&mut self, path: P, data: &[u8]) -> Result<()> {
        
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();

        self.archive.append_data(&mut header, path, data)
    }

    pub fn finish(self) -> Result<()> {

        let compressor = self.archive.into_inner()?;
        let mut obj = compressor.into_inner();

        obj.flush()
    }

}

