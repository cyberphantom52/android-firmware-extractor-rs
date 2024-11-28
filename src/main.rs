use clap::Parser;
use std::path::PathBuf;

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

fn main() {
    let args: Arguments = Arguments::parse();
}
