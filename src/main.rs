use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};
use dialoguer::{theme::ColorfulTheme, Select};

const NIX_BUILD_FHS: &'static str = "nix-build --no-out-link -E";
const LDD_NOT_FOUND: &'static str = " => not found";

fn fhs_shell(run: &Path, packages: Vec<String>) -> String {
    format!(
        r#"with import <nixpkgs> {{}};
  buildFHSUserEnv {{
    name = "fhs";
    targetPkgs = p: with p; [ 
      {} 
    ];
    runScript = "{}";
  }}"#,
        packages.join("\n      "),
        run.to_str().expect("unable to stringify path")
    )
}

fn make_shellscript(target: &Path, fhs_script: String) -> io::Result<()> {
    let mut file = File::create(target)?;
    file.write_all(
        format!(
            r#"#!/usr/bin/env bash

$({} '{}')/bin/fhs"#,
            NIX_BUILD_FHS, fhs_script,
        )
        .as_bytes(),
    )?;
    file.metadata()?.permissions().set_mode(0755);

    Ok(())
}

fn missing_libs(binary: &Path) -> Vec<String> {
    let output = Command::new("ldd")
        .arg(binary.to_str().expect("unable to stringify path"))
        .output()
        .expect("failed to execute ldd");

    if !output.status.success() {
        panic!("ldd returned error code {}", output.status);
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|l| match l.find(LDD_NOT_FOUND) {
            Some(i) => {
                let mut s = l.to_string();
                s.truncate(i);
                s.remove(0);
                Some(s.trim().to_string())
            }
            None => None,
        })
        .collect()
}

fn find_candidates(file_name: &String) -> Vec<(String, String)> {
    let output = Command::new("nix-locate")
        .arg("--top-level")
        .arg("--type=r")
        .arg("--type=s")
        .arg("--type=x")
        .arg("--whole-name")
        .arg(file_name)
        .output()
        .expect("failed to execute nix-locate");

    if !output.status.success() {
        panic!("nix-locate returned error code {}", output.status);
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| {
            let begin_cut = l.find(" ").unwrap();
            let end_cut = l.match_indices("/").skip(3).nth(0).unwrap().0;
            (l[0..begin_cut].to_string(), l[end_cut..].to_string())
        })
        .collect()
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("binary")
                .value_name("BINARY")
                .required(true)
                .help("dynamically linked binary to be examined"),
        )
        .arg(
            Arg::with_name("additional-libs")
                .short("l")
                .takes_value(true)
                .multiple(true)
                .help("Additional libraries to search and add"),
        )
        .arg(
            Arg::with_name("additional-packages")
                .short("p")
                .takes_value(true)
                .multiple(true)
                .help("Additional packages to add"),
        )
        .get_matches();

    let path_to_binary = Path::new(matches.value_of("binary").unwrap());

    let mut packages: Vec<String> = Vec::new();
    if let Some(additional_packages) = matches.values_of("additional-packages") {
        for p in additional_packages {
            packages.push(p.to_string());
        }
    }
    packages.dedup();
    packages.sort();

    let mut missing_libs = missing_libs(&path_to_binary);
    if let Some(additional_libs) = matches.values_of("additional-libs") {
        for p in additional_libs {
            missing_libs.push(p.to_string());
        }
    }
    missing_libs.dedup();
    missing_libs.sort();

    for lib in missing_libs {
        let candidates = find_candidates(&lib);
        match candidates.len() {
            0 => panic!("Found no provide for {}", lib),
            1 => packages.push(candidates[0].0.clone()),
            _ if candidates.iter().any(|c| packages.contains(&c.0)) => {}
            _ => {
                let selections: Vec<String> = candidates
                    .iter()
                    .map(|c| format!("{:32}{}", c.0, c.1))
                    .collect();
                let choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!("Pick provider for {}", lib))
                    .default(0)
                    .items(&selections[..])
                    .interact()
                    .unwrap();
                packages.push(candidates[choice].0.clone());
            }
        }
    }

    make_shellscript(
        &path_to_binary.with_file_name("run-with-nix"),
        fhs_shell(&path_to_binary.canonicalize().unwrap(), packages),
    )
    .unwrap();
}
