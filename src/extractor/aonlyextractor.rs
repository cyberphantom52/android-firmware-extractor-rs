use crate::utils::ZipFile;

use super::Extractable;

pub struct AOnlyExtractor(ZipFile);

impl From<ZipFile> for AOnlyExtractor {
    fn from(archive: ZipFile) -> Self {
        AOnlyExtractor(archive)
    }
}

impl Extractable for AOnlyExtractor {}
