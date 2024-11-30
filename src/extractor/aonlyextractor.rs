use crate::utils::ZipFile;
use regex::Regex;
use sdat2img_rs::{SparseDecoder, TransferList};
use std::{
    fs::File,
    io::Result,
    path::{Path, PathBuf},
};
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

    pub fn get_archived_partitions(&self, partition: &str) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let pattern = Regex::new(&format!(
            r"(?:.*\/)?{partition}\.(new\.dat.*|transfer\.list|img)$"
        ))
        .unwrap();
        let numbered_dat = Regex::new(r"\.new\.dat\.\d+$").unwrap();
        self.archive
            .files()
            .into_iter()
            .filter(|x| pattern.is_match(x.to_str().unwrap()))
            .partition(|x| !numbered_dat.is_match(x.to_str().unwrap()))
    }

    pub fn sdat2img(&self, transfer_list: &Path, sparse_file: &Path, output: &Path) -> Result<()> {
        let source = File::open(sparse_file)?;
        let destination = File::create(output)?;

        SparseDecoder::new(TransferList::try_from(transfer_list)?, source, destination)
            .quiet()
            .decode()
    }

    pub fn decompress(&self, compressed_file: &Path) -> Result<()> {
        let extension = compressed_file.extension().unwrap();
        let output = compressed_file
            .with_extension("")
            .file_name()
            .unwrap()
            .to_owned();
        let mut source = File::open(&compressed_file)?;
        let mut output_file = File::create(&self.tmpdir().join(output))?;

        if extension == "xz" {
            uncompress_data(&mut source, &mut output_file)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| ())?;
        } else {
            let mut decoder = Decompressor::new(source, 4096);
            std::io::copy(&mut decoder, &mut output_file)?;
        }

        std::fs::remove_file(&compressed_file)?;

        Ok(())
    }

    pub fn unsparse(&self, partition: &str, output: &Path) -> Result<()> {
        let sparse_file = self.tmpdir().join(format!("{}.new.dat", partition));
        let transfer_list = self.tmpdir().join(format!("{}.transfer.list", partition));
        let output_image = output.join(format!("{}.img", partition));

        self.sdat2img(&transfer_list, &sparse_file, &output_image)
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
        let (parts, mut numbered_dat_parts) = self.get_archived_partitions(partition);

        for part in parts {
            let destination = File::create(self.tmpdir().join(&part.file_name().unwrap()))?;
            self.archive.extract(&part, destination)?;
        }

        if !numbered_dat_parts.is_empty() {
            numbered_dat_parts.sort();
            let destination = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(self.tmpdir().join(format!("{partition}.new.dat")))?;
            for part in numbered_dat_parts {
                self.archive.extract(&part, &destination)?;
            }
        }

        for entry in std::fs::read_dir(self.tmpdir())? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.ends_with(".img") {
                    // Move the file to the output directory
                    std::fs::rename(&path, output.join(file_name))?;
                    return Ok(());
                }

                if file_name.ends_with(".xz") || file_name.ends_with(".br") {
                    self.decompress(&path)?;
                }
            }
        }

        let new_dat_path = self.tmpdir().join(format!("{}.new.dat", partition));
        if new_dat_path.exists() {
            let transfer_list = self.tmpdir().join(format!("{}.transfer.list", partition));
            if !transfer_list.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Could not find transfer list file for {}.new.dat",
                        partition
                    ),
                ));
            }

            self.unsparse(partition, output)?;
        }

        Ok(())
    }
}
