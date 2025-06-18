// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    env,
    sync::mpsc::{self, Receiver, Sender},
};

use anyhow::Result;
use chrono::Utc;
use env_filter::{Builder, FilteredLog};
use log::{Level, Record};
use systemd_journal_logger::{JournalLog, connected_to_journal};

type Message = (Level, Option<String>, String);

pub(crate) struct Logger {
    tx: Sender<Message>,
}

fn log_to_journal(rx: Receiver<Message>, logger: JournalLog, to_journal: bool) -> Result<()> {
    while let Ok((level, path, message)) = rx.recv() {
        // this is a hack to get around the lifetime limitations on [format_args!]
        #[allow(clippy::redundant_closure_call)]
        (|args| {
            let record = Record::builder()
                .level(level)
                .module_path(path.as_deref())
                .args(args)
                .build();
            if to_journal {
                if let Err(err) = logger.journal_send(&record) {
                    eprintln!(
                        "failed to log ({err:?}: [{} {} {}] {}",
                        Utc::now(),
                        record.level(),
                        record.module_path().unwrap_or_default(),
                        record.args()
                    );
                }
            } else {
                eprintln!(
                    "[{} {} {}] {}",
                    Utc::now(),
                    record.level(),
                    record.module_path().unwrap_or_default(),
                    record.args()
                );
            }
        })(format_args!("{message}"));
    }

    Ok(())
}

impl Logger {
    pub(crate) fn setup() -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let logger = JournalLog::new()?
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .with_syslog_identifier(env!("CARGO_PKG_NAME").to_string());
        let filter = Builder::from_env("RUST_LOG").build();

        log::set_boxed_logger(Box::new(FilteredLog::new(Logger { tx }, filter)))?;
        log::set_max_level(log::LevelFilter::Trace);

        std::thread::spawn(move || log_to_journal(rx, logger, connected_to_journal()));

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.tx
            .send((
                record.level(),
                record.module_path().map(|path| path.to_string()),
                record.args().to_string(),
            ))
            .expect("logging succeeds")
    }

    fn flush(&self) {}
}
