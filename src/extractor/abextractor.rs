use super::Extractable;
use crate::utils::ZipFile;
use payload_dumper_rs::Payload;
use std::io::Result;
use std::path::Path;
use temp_dir::TempDir;

pub struct ABExtractor {
    payload: Payload,
    _tmpdir: TempDir,
}

impl Drop for ABExtractor {
    fn drop(&mut self) {}
}

impl TryFrom<ZipFile> for ABExtractor {
    type Error = std::io::Error;

    fn try_from(archive: ZipFile) -> std::result::Result<Self, Self::Error> {
        let _tmpdir = TempDir::new()?;

        let payload_path = archive.extract("payload.bin", &_tmpdir.path())?;
        let payload = Payload::try_from(payload_path.as_path())?;
        Ok(ABExtractor { payload, _tmpdir })
    }
}

impl Extractable for ABExtractor {
    fn extract(&self, partition: &str, output: &Path) -> Result<()> {
        self.payload.extract(partition, output)
    }
}
