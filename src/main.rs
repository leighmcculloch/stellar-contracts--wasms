use clap::{Parser, ValueEnum};
use std::{
    io::{BufReader, Write},
    process::{Command, Stdio},
};
use stellar_xdr::curr::{Frame, LedgerCloseMeta, Limited, Limits, ReadXdr};
use tempfile::NamedTempFile;

#[derive(Parser)]
struct Args {
    /// Path to stellar-core binary
    #[arg(long, default_value = "stellar-core")]
    stellar_core_path: String,

    /// Path to stellar-core configuration file
    #[arg(long, default_value = "testnet")]
    network: Network,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Network {
    Testnet,
    Pubnet,
}

impl Network {
    pub fn config(&self) -> &'static str {
        match self {
            Network::Testnet => include_str!("stellar-core-testnet.cfg"),
            Network::Pubnet => include_str!("stellar-core-pubnet.cfg"),
        }
    }

    pub fn config_file(&self) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", self.config()).unwrap();
        file
    }
}

fn main() {
    let args = Args::parse();
    let config_file = args.network.config_file();

    let mut core = Command::new(&args.stellar_core_path)
        .arg("run")
        .arg("--conf")
        .arg(config_file.path())
        .arg("--metadata-output-stream")
        .arg("fd:1")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = core.stdout.take().unwrap();
    let buffer = BufReader::new(stdout);
    let mut limited = Limited::new(buffer, Limits::none());
    let iter = Frame::<LedgerCloseMeta>::read_xdr_iter(&mut limited);
    for frame in iter {
        let Frame(meta) = frame.unwrap();
        let json = serde_json::to_string(&meta).unwrap();
        println!("{json}");
    }

    core.wait().unwrap();
}
