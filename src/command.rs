use crate::componentize;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{ffi::OsString, path::PathBuf};

/// A tool that creates Go WebAssembly components.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    #[command(flatten)]
    pub common: Common,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Clone, Debug)]
pub struct Common {
    /// The location of the WIT document(s).
    ///
    /// This may be specified more than once, for example:
    /// `-d ./wit/deps -d ./wit/app`
    #[arg(long, short = 'd')]
    pub wit_path: Vec<PathBuf>,

    /// Name of world to target (or default world if `None`).
    #[arg(long, short = 'w')]
    pub world: Option<String>,

    /// Whether or not to activate all WIT features when processing WIT files.
    ///
    /// This enables using `@unstable` annotations in WIT files.
    #[arg(long)]
    pub all_features: bool,

    /// Comma-separated list of features that should be enabled when processing
    /// WIT files.
    ///
    /// This enables using `@unstable` annotations in WIT files.
    #[arg(long)]
    pub features: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build a Go WebAssembly component.
    Componentize(Componentize),
}

#[derive(Parser)]
pub struct Componentize {
    /// The path to the Go binary (or look for binary in PATH if `None`).
    #[arg(long)]
    pub go: Option<PathBuf>,

    /// Final output path for the component (or `./main.wasm` if `None`).
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// The directory containing the "go.mod" file (or current directory if `None`).
    #[arg(long = "mod")]
    pub mod_path: Option<PathBuf>,
}

pub fn run<T: Into<OsString> + Clone, I: IntoIterator<Item = T>>(args: I) -> Result<()> {
    let options = Options::parse_from(args);
    match options.command {
        Command::Componentize(opts) => componentize(options.common, opts),
    }
}

fn componentize(common: Common, componentize: Componentize) -> Result<()> {
    // Step 1: Build a WebAssembly core module using Go.
    let core_module = componentize::build_wasm_core_module(
        componentize.mod_path,
        componentize.output,
        componentize.go,
    )?;

    // Step 2: Embed the WIT documents in the core module.
    componentize::embed_wit(
        &core_module,
        &common.wit_path,
        common.world.as_deref(),
        &common.features,
        common.all_features,
    )?;

    // Step 3: Update the core module to use the component model ABI.
    componentize::core_module_to_component(&core_module)?;
    Ok(())
}
