use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
pub struct ZipFile(File);

const ZIP_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

impl TryFrom<&Path> for ZipFile {
    type Error = std::io::Error;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let file = File::open(value)?;

        // Verify file signature
        let mut buf = [0; 4];
        file.read_exact_at(&mut buf, 0)?;

        if buf != ZIP_SIGNATURE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Input is not a valid ZIP file.",
            ));
        }

        Ok(ZipFile(file))
    }
}

impl ZipFile {
    pub fn get_archived_basenames(&self) -> Vec<String> {
        compress_tools::list_archive_files(&self.0)
            .unwrap()
            .into_iter()
            .map(|file| file.split('/').last().unwrap().to_string())
            .collect()
    }

    pub fn get_relative_path(&self, file: &str) -> Option<String> {
        compress_tools::list_archive_files(&self.0)
            .unwrap()
            .into_iter()
            .find(|f| f.ends_with(file))
    }

    pub fn extract(&self, file: &str, output_dir: &Path) -> std::io::Result<()> {
        let relative_path = self.get_relative_path(file).unwrap();
        let output_file = File::create(output_dir.join(&relative_path))?;
        compress_tools::uncompress_archive_file(&self.0, output_file, &relative_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .map(|_| ())
    }
}

pub fn default_output_path(firmware_zip_path: &Path) -> PathBuf {
    let base_dir = firmware_zip_path.parent().unwrap();
    let time_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    base_dir.join(format!("output-{}", time_now.as_secs()))
}
