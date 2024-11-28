use crate::utils::ZipFile;

use super::Extractable;

pub struct ABExtractor(ZipFile);

impl From<ZipFile> for ABExtractor {
    fn from(archive: ZipFile) -> Self {
        ABExtractor(archive)
    }
}

impl Extractable for ABExtractor {}
