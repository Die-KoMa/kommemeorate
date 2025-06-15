// SPDX-FileCopyrightText: © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Cli {
    config: Option<PathBuf>,
}
