use crate::utils::ZipFile;

use super::Extractable;

pub struct SparseChunkExtractor(ZipFile);

impl From<ZipFile> for SparseChunkExtractor {
    fn from(archive: ZipFile) -> Self {
        SparseChunkExtractor(archive)
    }
}

impl Extractable for SparseChunkExtractor {}
