// Copyright 2022 Andris Ļaksa (linkedin.com/in/andris-laksa)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cargo::core::Workspace;
use cargo::ops::load_pkg_lockfile;
use cargo::util::short_hash;
use cargo::Config;
use clap::{Parser, Subcommand};
use log::{info, LevelFilter};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct JfrogDownloadFile {
    pattern: String,
    target: String,
    flat: String,
}

#[derive(Serialize)]
struct JfrogDownloadSpec {
    files: Vec<JfrogDownloadFile>,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(name = "jfrog-dl-spec")]
    Main {
        /// Name of the cargo registry as mentioned in Cargo.toml
        #[clap(short, long)]
        registry: String,

        /// Download only crates missing from cache
        #[clap(short, long)]
        missing_only: bool,
    },
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();
    let (registry, missing_only) = match args.command {
        Command::Main {
            registry,
            missing_only,
        } => (registry, missing_only),
    };

    let config = Config::default().unwrap();
    let path = Path::new("./Cargo.toml").canonicalize().unwrap();
    let ws = Workspace::new(&path, &config).unwrap();
    let res = load_pkg_lockfile(&ws).unwrap().unwrap();

    let mut spec = JfrogDownloadSpec { files: vec![] };

    for dep in res
        .iter()
        .filter(|d| d.source_id().display_registry_name() == registry)
    {
        let reg_name = format!(
            "{}-{}",
            dep.source_id().url().host().unwrap(),
            short_hash(&dep.source_id())
        );

        let cache_path = config.registry_cache_path().join(reg_name);

        if cache_path.as_path_unlocked().exists() && missing_only {
            info!("Crate {} exists in cache, skipping...", dep);
            continue;
        }

        spec.files.push(JfrogDownloadFile {
            pattern: format!(
                "{repo}/crates/{name}/{name}-{version}.crate",
                repo = Path::new(dep.source_id().url().path())
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                name = dep.name(),
                version = dep.version()
            ),
            target: format!("{}/", cache_path.display()),
            flat: "true".to_string(),
        });
    }

    info!("{} crate(s) to download", spec.files.len());
    println!("{}", serde_json::to_string(&spec).unwrap());
}
