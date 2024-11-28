use crate::utils::ZipFile;

use super::Extractable;

pub struct ApTarMd5Extractor(ZipFile);

impl From<ZipFile> for ApTarMd5Extractor {
    fn from(archive: ZipFile) -> Self {
        ApTarMd5Extractor(archive)
    }
}

impl Extractable for ApTarMd5Extractor {}
