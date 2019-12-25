use std::fs;
use std::io;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use clap::{App, Arg};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Select};

const NIX_BUILD_FHS: &'static str = "nix-build --no-out-link -E";
const LDD_NOT_FOUND: &'static str = " => not found";

/// Writes a shellscript
fn write_bash_script(target: &Path, script: &String) -> io::Result<()> {
    let mut file = fs::File::create(target)?;
    file.write_all(format!("#!/usr/bin/env bash\n\n{}", script).as_bytes())?;

    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions)?;

    Ok(())
}

/// Returns the nix expression needed to build an appropiate FHS
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

/// uses ldd to find missing shared object files on a given binary
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
                s.remove(0); // get rid of tabulator prefix
                Some(s.trim().to_string())
            }
            None => None,
        })
        .collect()
}

/// uses nix-locate to find candidate packages providing a given file,
/// identified by a file name
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
            Arg::with_name("libs")
                .short("l")
                .long("additional-libs")
                .takes_value(true)
                .multiple(true)
                .help("Additional libraries to search for and propagate"),
        )
        .arg(
            Arg::with_name("packages")
                .short("p")
                .long("additional-pkgs")
                .takes_value(true)
                .multiple(true)
                .help("Additional packages to propagate"),
        )
        .get_matches();

    // the binary to be processed
    let path_to_binary = Path::new(matches.value_of("binary").unwrap());

    // initilizes packages list and adds additional-packages right away, if
    // provided
    let mut packages: Vec<String> = Vec::new();
    if let Some(additional_packages) = matches.values_of("packages") {
        for p in additional_packages {
            packages.push(p.to_string());
        }
    }
    packages.dedup();
    packages.sort();

    let mut missing_libs = missing_libs(&path_to_binary);
    if let Some(additional_libs) = matches.values_of("libs") {
        for p in additional_libs {
            missing_libs.push(p.to_string());
        }
    }
    missing_libs.dedup();
    missing_libs.sort();

    // using two thread producer/consumer architecture. This reduces waiting
    // time by a lot, as nix-locate calls are really time expensive
    let (sender, receiver) = channel();
    thread::spawn(move || {
        for lib in missing_libs {
            let candidates = find_candidates(&lib);
            sender.send((lib, candidates)).unwrap();
        }
    });

    loop {
        if let Ok((lib, candidates)) = receiver.recv() {
            match candidates.len() {
                0 => panic!("Found no provide for {}", lib),
                1 => packages.push(candidates[0].0.clone()),
                _ if candidates.iter().any(|c| packages.contains(&c.0)) => {}
                _ => {
                    let bold = Style::new().bold().red();
                    let selections: Vec<String> = candidates
                        .iter()
                        .map(|c| format!("{} {}", c.0, c.1))
                        .collect();
                    let choice = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(&format!("Pick provider for {}", bold.apply_to(lib)))
                        .default(0)
                        .items(&selections[..])
                        .interact()
                        .unwrap();
                    packages.push(candidates[choice].0.clone());
                }
            }
        } else {
            break;
        }
    }

    // build FHS expression
    let fhs_expression = fhs_shell(&path_to_binary.canonicalize().unwrap(), packages);
    // write bash script with the FHS expression
    write_bash_script(
        &path_to_binary.with_file_name("run-with-nix"),
        &format!("$({} '{}')/bin/fhs", NIX_BUILD_FHS, fhs_expression),
    )
    .unwrap();
}
