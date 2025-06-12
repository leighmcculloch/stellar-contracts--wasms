use clap::{Parser, ValueEnum};
use colored::Colorize as _;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufRead, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};
use stellar_xdr::curr::{
    Frame, LedgerCloseMeta, LedgerEntryChange, LedgerEntryData, Limited, Limits, ReadXdr,
    TransactionMeta,
};

#[derive(Parser)]
struct Args {
    /// Network to connect to
    #[arg(long, default_value = "testnet")]
    network: Network,

    /// Ledger to start at
    #[arg(long)]
    ledger: Option<u32>,

    /// Path to stellar-core binary
    #[arg(long, default_value = "stellar-core")]
    stellar_core_path: OsString,

    /// Path to use as the stellar-core working directory
    #[arg(long)]
    stellar_core_working_dir: Option<PathBuf>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Network {
    Testnet,
    Pubnet,
}

fn main() {
    let args = Args::parse();
    let tempdir = tempfile::tempdir().unwrap();
    let working_dir = args
        .stellar_core_working_dir
        .as_ref()
        .map_or(tempdir.path(), |wd| wd.as_path());
    write_cfg_file(working_dir, args.network.config());

    let (send, recv) = mpsc::channel();
    enum Log {
        Out(String),
        Err(String),
    }
    thread::spawn(move || {
        for log in recv {
            match log {
                Log::Out(l) => println!("{l}"),
                Log::Err(l) => eprintln!("{}", l.bright_black()),
            }
        }
    });

    let mut core = Command::new(&args.stellar_core_path)
        .arg("new-db")
        .current_dir(&working_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = core.stderr.take().unwrap();
    let send_stderr = send.clone();
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            if let Ok(line) = line {
                let _ = send_stderr.send(Log::Err(line));
            }
        }
    });
    assert!(core.wait().unwrap().success());

    if let Some(ledger) = &args.ledger {
        let mut core = Command::new(&args.stellar_core_path)
            .args(["catchup", &format!("{}/0", ledger.saturating_sub(1))])
            .current_dir(&working_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stderr = core.stderr.take().unwrap();
        let send_stderr = send.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                if let Ok(line) = line {
                    let _ = send_stderr.send(Log::Err(line));
                }
            }
        });
        assert!(core.wait().unwrap().success());
    }

    let mut core = Command::new(&args.stellar_core_path)
        .args(["run", "--metadata-output-stream", "fd:1"])
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = core.stderr.take().unwrap();
    let send_stderr = send.clone();
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            if let Ok(line) = line {
                let _ = send_stderr.send(Log::Err(line));
            }
        }
    });
    let stdout = core.stdout.take().unwrap();
    let send_json = send.clone();
    thread::spawn(move || {
        let buffer = BufReader::new(stdout);
        let mut limited = Limited::new(buffer, Limits::none());
        let iter = Frame::<LedgerCloseMeta>::read_xdr_iter(&mut limited);
        for frame in iter {
            let Frame(meta) = frame.unwrap();

            //let json = serde_json::to_string(&meta).unwrap();
            //let _ = send_json.send(Log::Out(json));

            let (header, txs) = match &meta {
                LedgerCloseMeta::V0(v0) => (&v0.ledger_header, &v0.tx_processing),
                LedgerCloseMeta::V1(v1) => (&v1.ledger_header, &v1.tx_processing),
            };
            let ledger = header.header.ledger_seq;
            let changes = txs
                .iter()
                .flat_map(|tx| match &tx.tx_apply_processing {
                    TransactionMeta::V0(operations) => operations.iter(),
                    TransactionMeta::V1(v1) => v1.operations.iter(),
                    TransactionMeta::V2(v2) => v2.operations.iter(),
                    TransactionMeta::V3(v3) => v3.operations.iter(),
                })
                .flat_map(|op| op.changes.iter());

            let mut new = false;
            for change in changes {
                match change {
                    LedgerEntryChange::Created(ledger_entry) => match &ledger_entry.data {
                        LedgerEntryData::ContractCode(entry) => {
                            let hash = entry.hash.0;
                            let hash_str = hex::encode(hash);
                            let code = entry.code.as_slice();
                            fs::write(format!("{hash_str}.wasm"), code).unwrap();
                            let _ = send_json.send(Log::Out(hash_str));
                            new = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            if new {
                Command::new("git").args(["add", "."]).spawn().unwrap();
                Command::new("git")
                    .args(["commit", "-m", &format!("{}", ledger)])
                    .spawn()
                    .unwrap();
                Command::new("git").args(["push"]).spawn().unwrap();
            }
        }
    });

    assert!(core.wait().unwrap().success());
}

fn write_cfg_file(dir: &Path, config: &str) {
    let config_file_path = dir.join("stellar-core.cfg");
    let mut config_file = File::create(&config_file_path).unwrap();
    writeln!(config_file, "{}", config).unwrap();
}

impl Network {
    pub fn config(&self) -> &'static str {
        match self {
            Network::Testnet => include_str!("stellar-core-testnet.cfg"),
            Network::Pubnet => include_str!("stellar-core-pubnet.cfg"),
        }
    }
}
