use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, required = true)]
    pub mode: LicenseType,
    #[arg(short, long, default_value = "drivers.csv")]
    pub drivers: String,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum LicenseType {
    Road,
    Oval,
    DirtOval,
    DirtRoad,
}

pub fn parse() -> Args {
    Args::parse()
}
