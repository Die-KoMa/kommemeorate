// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate_to};
use clap_mangen::Man;
use std::env;
use std::fs::File;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let real_outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let outdir = match env::var_os("MAN_OUT") {
        None => real_outdir,
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "kommemeorate", &outdir)?;
    }

    let file = PathBuf::from(&outdir).join("kommemeorate.1");
    let mut file = File::create(file)?;

    Man::new(cmd).render(&mut file)?;

    println!("cargo:infog=completion files & manpages are generated: {outdir:?}");

    Ok(())
}
