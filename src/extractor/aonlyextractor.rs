use crate::utils::ZipFile;
use regex::Regex;
use sdat2img_rs::{SparseDecoder, TransferList};
use std::{fs::File, io::Result, path::Path};
use temp_dir::TempDir;
use {brotli::Decompressor, compress_tools::uncompress_data};

use super::Extractable;

pub struct AOnlyExtractor {
    archive: ZipFile,
    tmpdir: TempDir,
}

impl AOnlyExtractor {
    pub fn tmpdir(&self) -> &Path {
        self.tmpdir.path()
    }
}

impl From<ZipFile> for AOnlyExtractor {
    fn from(archive: ZipFile) -> Self {
        AOnlyExtractor {
            archive,
            tmpdir: TempDir::new().unwrap(),
        }
    }
}

impl Extractable for AOnlyExtractor {
    fn extract(&self, partition: &str, output: &Path) -> Result<()> {
        let pattern =
            Regex::new(&format!(r"^{partition}\.(img|transfer\.list|new\.dat.*)$")).unwrap();
        let numbered_dat = Regex::new(r"\.new\.dat\.\d+$").unwrap();
        let (parts, mut numbered_dat_parts): (Vec<_>, Vec<_>) = self
            .archive
            .get_archived_basenames()
            .into_iter()
            .filter(|x| pattern.is_match(x))
            .partition(|x| !numbered_dat.is_match(x));

        for part in parts {
            self.archive.extract(&part, self.tmpdir())?;
        }

        if !numbered_dat_parts.is_empty() {
            numbered_dat_parts.sort();
            let dat_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(self.tmpdir().join(format!("{partition}.new.dat")))?;
            for part in numbered_dat_parts {
                self.archive.extract_to_file(&part, &dat_file)?;
            }
        }

        for entry in std::fs::read_dir(output)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.ends_with(".img") {
                    // Move the file to the output directory
                    std::fs::rename(
                        self.tmpdir().join(&path.file_name().unwrap()),
                        output.join(&path.file_name().unwrap()),
                    )?;
                    return Ok(());
                }

                if file_name.ends_with(".dat.xz") {
                    let mut compressed_file = File::open(&path)?;
                    let mut output_file =
                        File::create(&self.tmpdir().join(file_name.strip_suffix(".xz").unwrap()))?;

                    uncompress_data(&mut compressed_file, &mut output_file)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                        .map(|_| ())?;

                    std::fs::remove_file(&path)?;
                } else if file_name.ends_with(".dat.br") {
                    let compressed_file = File::open(&path)?;
                    let mut output_file =
                        File::create(&self.tmpdir().join(file_name.strip_suffix(".br").unwrap()))?;

                    let mut decoder = Decompressor::new(compressed_file, 4096);
                    std::io::copy(&mut decoder, &mut output_file)?;

                    std::fs::remove_file(&path)?;
                }
            }
        }

        let new_dat_path = self.tmpdir().join(format!("{}.new.dat", partition));

        if new_dat_path.exists() {
            let transfer_list_path = self.tmpdir().join(format!("{}.transfer.list", partition));
            let image_path = output.join(format!("{}.img", partition));

            if !transfer_list_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Could not find transfer list file for {}.new.dat",
                        partition
                    ),
                ));
            }

            let source = File::open(new_dat_path)?;
            let dest = File::open(image_path)?;
            SparseDecoder::new(
                TransferList::try_from(transfer_list_path.as_path())?,
                source,
                dest,
            )
            .decode()?;
        }

        Ok(())
    }
}
