// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde_json::Value;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const MANIFEST_JSON_URL: &str =
    "https://github.com/MystenLabs/sui/raw/mainnet/crates/sui-framework-snapshot/manifest.json";

// 定义与JSON结构匹配的数据模型
#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    path: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct VersionEntry {
    git_revision: String,
    packages: Vec<Package>,
}

/// 使用curl从远程拉取最新的system packages JSON并解析（假设按顺序排列，取最后一个）
fn fetch_latest_system_packages() -> anyhow::Result<Option<(u32, VersionEntry)>> {
    use std::process::Command;
    
    println!("Fetching manifest JSON with curl");
    let output = Command::new("curl")
        .arg("-s")
        .arg("-L") // Follow redirects
        .arg(MANIFEST_JSON_URL)
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("curl command failed"));
    }
    
    let json_str = String::from_utf8(output.stdout)?;
    let json_data: Value = serde_json::from_str(&json_str)?;
    
    if let Value::Object(map) = json_data {
        let mut entries: Vec<(String, Value)> = map.into_iter().collect();
        if let Some((last_key, last_value)) = entries.pop() {
            if let Ok(version) = last_key.parse::<u32>() {
                let entry: VersionEntry = serde_json::from_value(last_value)?;
                return Ok(Some((version, entry)));
            }
        }
    }
    
    Ok(None)
}

fn generate_system_packages_version_table() -> anyhow::Result<()> {
    let (latest_version, latest_entry) = match fetch_latest_system_packages()? {
        Some(data) => data,
        None => return Err(anyhow::anyhow!("fetch_latest_system_packages failed.")),
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("system_packages_version_table.rs");
    let mut file = BufWriter::new(File::create(&dest_path)?);

    writeln!(&mut file, "[")?;
    writeln!(
        &mut file,
        "  (ProtocolVersion::new( {latest_version:>2} ), SystemPackagesVersion {{"
    )?;
    writeln!(
        &mut file,
        "        git_revision: \"{}\".into(),",
        latest_entry.git_revision
    )?;
    writeln!(&mut file, "        packages: [")?;

    for package in latest_entry.packages.iter() {
        writeln!(
            &mut file,
            "          SystemPackage {{ package_name: \"{}\".into(), repo_path: \"{}\".into(), id: \"{}\".into() }},",
            package.name,
            package.path,
            package.id
        )?;
    }

    writeln!(&mut file, "        ].into(),")?;
    writeln!(&mut file, "      }}),")?;
    writeln!(&mut file, "]")?;

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo:rustc-env=SUI_SYS_PKG_TABLE={}", dest_path.display());
    Ok(())
}

fn main() {
    generate_system_packages_version_table().unwrap();
}
