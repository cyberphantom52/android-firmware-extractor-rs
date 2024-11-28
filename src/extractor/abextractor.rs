use super::Extractable;
use crate::utils::ZipFile;
use payload_dumper_rs::Payload;
use std::env::temp_dir;
use std::io::Result;
use std::path::Path;

pub struct ABExtractor(Payload);

impl Drop for ABExtractor {
    fn drop(&mut self) {}
}

impl TryFrom<ZipFile> for ABExtractor {
    type Error = std::io::Error;

    fn try_from(archive: ZipFile) -> std::result::Result<Self, Self::Error> {
        let tmpdir = temp_dir().join("android_firmware_extractor");

        if !tmpdir.exists() {
            std::fs::create_dir_all(&tmpdir)?;
        }

        let payload_path = archive.extract("payload.bin", &tmpdir)?;
        let payload = Payload::try_from(payload_path.as_path())?;
        Ok(ABExtractor(payload))
    }
}

impl Extractable for ABExtractor {
    fn extract(&self, partition: &str, output: &Path) -> Result<()> {
        self.0.extract(partition, output)
    }
}
