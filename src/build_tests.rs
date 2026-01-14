use crate::common::{check_go_version, make_path_absolute};
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process::Command};

pub fn build_unit_test_modules(
    module_path: Option<PathBuf>,
    packages: Vec<PathBuf>,
    out: Option<PathBuf>,
    go_path: Option<PathBuf>,
) -> Result<()> {
    if packages.is_empty() {
        return Err(anyhow!("you must specify at least one package"));
    }

    let go = match &go_path {
        Some(p) => make_path_absolute(p)?,
        None => PathBuf::from("go"),
    };

    check_go_version(&go)?;

    let out_path_buf = match &out {
        Some(p) => make_path_absolute(p)?,
        None => std::env::current_dir()?.join("wasm_testfiles"),
    };

    std::fs::create_dir_all(&out_path_buf)?;

    let module_path = match &module_path {
        Some(p) => {
            if !p.is_dir() {
                return Err(anyhow!("Module path '{}' is not a directory", p.display()));
            }
            p
        }
        None => &std::env::current_dir()?,
    };

    for package in packages.iter() {
        let out = out_path_buf.join(format!("{}.wasm", package.to_str().unwrap()));
        let pkg = if package.is_relative() {
            module_path.join(package)
        } else {
            package.to_owned()
        };

        let output = Command::new(&go)
            .args([
                "test",
                "-c",
                "-ldflags=-checklinkname=0",
                "-o",
                out.to_str().expect("Wasm out path is not valid unicode"),
                pkg.to_str().expect("Package path is not valid unicode"),
            ])
            .env("GOOS", "wasip1")
            .env("GOARCH", "wasm")
            .current_dir(module_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "'go test' command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}
