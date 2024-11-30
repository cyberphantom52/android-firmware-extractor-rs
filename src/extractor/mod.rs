use crate::utils::ZipFile;
use std::path::Path;
mod abextractor;
mod aonlyextractor;
mod aptarmd5extractor;
mod sparsechunkextractor;

use {
    abextractor::ABExtractor, aonlyextractor::AOnlyExtractor, aptarmd5extractor::ApTarMd5Extractor,
    sparsechunkextractor::SparseChunkExtractor,
};

pub trait Extractable {
    fn extract(&self, partition: &str, output: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }
}

pub struct Extractor {
    extractor: Box<dyn Extractable>,
}

impl Extractable for Extractor {
    fn extract(&self, partition: &str, output: &Path) -> std::io::Result<()> {
        self.extractor.extract(partition, output)
    }
}

impl TryFrom<ZipFile> for Extractor {
    type Error = std::io::Error;

    fn try_from(archive: ZipFile) -> std::result::Result<Self, Self::Error> {
        let files = archive.get_archived_basenames();

        if files.iter().any(|file| file == "system.new.dat") {
            println!("A Only Firmware Detected");
            return Ok(Extractor {
                extractor: Box::new(AOnlyExtractor::from(archive)),
            });
        }

        if files.iter().any(|file| file == "payload.bin") {
            println!("A/B Firmware Detected");
            let extractor = ABExtractor::try_from(archive)?;
            return Ok(Extractor {
                extractor: Box::new(extractor),
            });
        }

        if files
            .iter()
            .any(|file| file.starts_with("AP_") && file.ends_with("tar.md5"))
        {
            println!("Samsung Firmware Detected");
            return Ok(Extractor {
                extractor: Box::new(ApTarMd5Extractor::from(archive)),
            });
        }

        if files.iter().any(|file| file.contains("sparsechunk")) {
            println!("Sparse Chunk Firmware Detected");
            return Ok(Extractor {
                extractor: Box::new(SparseChunkExtractor::from(archive)),
            });
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unsupported firmware file",
        ))
    }
}
