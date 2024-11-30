use clap::Parser;
use extractor::Extractable;
use std::path::PathBuf;
mod extractor;
pub mod utils;

#[derive(Parser, Debug)]
#[command(
    author = "Inam Ul Haq",
    version = "1.0",
    about = "Android Firmware Extractor"
)]
struct Arguments {
    firmware_zip_path: PathBuf,

    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    #[arg(short = 'p', long = "partitions", value_delimiter = ',')]
    partitions: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args: Arguments = Arguments::parse();
    let output = args
        .output
        .unwrap_or_else(|| utils::default_output_path(&args.firmware_zip_path));
    let archive = utils::ZipFile::try_from(args.firmware_zip_path.as_path())?;
    let extractor = extractor::Extractor::try_from(archive)?;

    for partition in args.partitions.iter() {
        extractor.extract(partition, output.as_path())?;
    }
    Ok(())
}
