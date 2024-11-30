use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
pub struct ZipFile(File);

const ZIP_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

impl TryFrom<&Path> for ZipFile {
    type Error = std::io::Error;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let mut file = File::open(value)?;

        // Verify file signature
        let mut buf = [0; 4];
        file.read_exact(&mut buf)?;

        if buf != ZIP_SIGNATURE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Input is not a valid ZIP file.",
            ));
        }
        file.rewind()?;

        Ok(ZipFile(file))
    }
}

impl ZipFile {
    /// Returns a list of files in the archive.
    ///
    /// The files are returned as [PathBuf] objects.
    pub fn files(&self) -> Vec<PathBuf> {
        compress_tools::list_archive_files(&self.0)
            .unwrap()
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }

    /// Returns a list of file names in the archive without the relative path.
    pub fn file_names(&self) -> Vec<String> {
        self.files()
            .into_iter()
            .map(|file| file.file_name().unwrap().to_str().unwrap().to_string())
            .collect()
    }

    /// Extracts a file from the archive to the specified destination.
    ///
    /// # Arguments
    ///
    /// * `archived_file_path` - The path to the file in the archive.
    ///
    /// * `destination` - The destination to extract the file to.
    ///
    /// # Returns
    ///
    /// Returns an error if the file cannot be extracted.
    pub fn extract<W: Write>(
        &self,
        archived_file_path: &Path,
        destination: W,
    ) -> std::io::Result<()> {
        compress_tools::uncompress_archive_file(
            &self.0,
            destination,
            archived_file_path.to_str().unwrap(),
        )
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
