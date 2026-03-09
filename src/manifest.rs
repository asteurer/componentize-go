use anyhow::{Error, anyhow};
use std::{collections::HashMap, fs, path::PathBuf, process::Command, str::FromStr};

use serde::Deserialize;

use crate::utils::make_path_absolute;

#[derive(Deserialize)]
struct RawComponentizeGoConfig {
    wit: Vec<WitConfig>,
}

#[derive(Deserialize)]
struct WitConfig {
    path: String,
    import_interface_names: Vec<String>,
    export_interface_names: Vec<String>,
}

// struct RawComponentizeGoConfig {
//     bindings: Option<String>,
//     wit_directory: Option<String>,
//     #[serde(default)]
//     import_interface_names: HashMap<String, String>,
//     #[serde(default)]
//     export_interface_names: HashMap<String, String>,
// }

// #[derive(Debug)]
// struct ComponentizeGoConfig {
//     bindings: Option<PathBuf>,
//     wit_directory: Option<PathBuf>,
//     import_interface_names: HashMap<String, String>,
//     export_interface_names: HashMap<String, String>,
// }

pub fn search_go_mod_deps(
    go_module: Option<&PathBuf>,
    go_path: Option<&PathBuf>,
) -> Result<(), Error> {
    let module_path = match &go_module {
        Some(p) => {
            if !p.is_dir() {
                return Err(anyhow!("Module path '{}' is not a directory", p.display()));
            }
            &format!(
                "{}/go.mod",
                p.to_str()
                    .ok_or_else(|| anyhow!("Module path is not valid unicode"))?
            )
        }
        None => "./go.mod",
    };

    let go = match &go_path {
        Some(p) => make_path_absolute(p)?,
        None => PathBuf::from("go"),
    };

    let mod_file_bytes = fs::read(module_path)?;
    let mod_file_string = String::from_utf8_lossy(&mod_file_bytes);
    let parsed_mod = gomod_parser::GoMod::from_str(&mod_file_string).map_err(|e| anyhow!(e))?;

    let exclusions: HashMap<&str, ()> = parsed_mod
        .exclude
        .iter()
        .map(|v| (v.module.module_path.as_str(), ()))
        .collect();

    let requirements = parsed_mod
        .require
        .iter()
        .map(|dep| {
            locate_manifest(
                &dep.module.module_path,
                Some(&dep.module.version),
                &go,
                &exclusions,
            )
        })
        .filter_map(|dep| dep.transpose())
        .collect::<Result<Vec<PathBuf>, Error>>()?;

    let replacements = parsed_mod
        .replace
        .iter()
        .map(|rep| {
            let (replacement, version) = match &rep.replacement {
                gomod_parser::Replacement::FilePath(p) => (p, None),
                gomod_parser::Replacement::Module(m) => (&m.module_path, Some(m.version.clone())),
            };
            locate_manifest(replacement, version.as_deref(), &go, &exclusions)
        })
        .filter_map(|rep| rep.transpose())
        .collect::<Result<Vec<PathBuf>, Error>>()?;

    let manifests = [requirements.as_slice(), replacements.as_slice()].concat();

    parse_manifests(&manifests)?;

    Ok(())
}

/// Use the `go list` command to locate a go.mod dependency that has
/// componentize-go manifest file in their root.
fn locate_manifest(
    dep_path: &str,
    dep_version: Option<&str>,
    go_path: &PathBuf,
    exclusions: &HashMap<&str, ()>,
) -> Result<Option<PathBuf>, Error> {
    if exclusions.contains_key(dep_path) {
        // Skip excluded modules
        return Ok(None);
    }

    let version = if let Some(v) = dep_version {
        &format!("@{v}")
    } else {
        ""
    };

    // Retrieve the absolute path to the directory housing the module
    let go_list_output = Command::new(go_path)
        .args([
            "list",
            "-mod=readonly",
            "-m",
            "-f",
            "'{{.Dir}}'",
            &format!("{dep_path}{version}"),
        ])
        .output()?;

    let module_to_try = if go_list_output.status.success() {
        PathBuf::from(String::from_utf8_lossy(&go_list_output.stdout).to_string())
    } else {
        return Err(anyhow!(
            "'go list' command failed: {}",
            String::from_utf8_lossy(&go_list_output.stderr)
        ));
    };

    if module_to_try.join("componentize-go.toml").exists() {
        Ok(Some(module_to_try))
    } else {
        Ok(None)
    }
}

fn parse_manifests(manifests: &[PathBuf]) -> Result<(), Error> {
    Ok(())
}
