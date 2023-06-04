extern crate bindgen;

use glob::glob;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Constant {
    name: String,
    value: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Event {
    name: String,
    encoding: String,
    desc: Option<String>,
    precise: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Arch {
    name: String,
    constants: Vec<Constant>,
    events: Vec<Event>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ArchFile {
    arch: Arch,
}

fn main() {
    let bindings = bindgen::Builder::default()
        .header("interop/cpp/include/pmu/pmu_enums.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let mut archs: String = "".to_owned();

    for entry in glob("src/events/**/*.yaml").expect("Failed to glob configs") {
        match entry {
            Ok(path) => {
                println!("cargo:rerun-if-changed={}", path.display());
                let file_txt = std::fs::read_to_string(path).expect("Failed to read path");
                let arch_file: ArchFile =
                    serde_yaml::from_str(&file_txt).expect("Failed to parse yaml");
                let arch = &arch_file.arch;
                archs.push_str(&format!("mod {} {{\n", &arch.name));
                archs += "  use crate::{SystemCounter, SystemCounterKind};\n";
                for cst in &arch.constants {
                    archs.push_str(&format!("  const {}: u64 = {};\n", &cst.name, cst.value));
                }
                let mut evt_names = vec![];
                for evt in &arch.events {
                    let const_name = evt.name.replace(".", "_");
                    evt_names.push(const_name.clone());
                    archs.push_str(&format!(
                        "  const {}: SystemCounter = SystemCounter {{
    kind: SystemCounterKind::Hardware,
    name: \"{}\",
    encoding: {},
  }};\n",
                        &const_name, &evt.name, &evt.encoding
                    ));
                }

                archs += "  pub(crate) fn get() -> Vec<crate::SystemCounter> {\n    vec![\n";
                for name in evt_names {
                    archs.push_str(&format!("      {}.clone(),\n", &name));
                }
                archs += "    ]\n  }\n";
                archs += "}\n";
            }
            Err(_) => {
                panic!("Failed");
            }
        };
    }

    let mut arch_file =
        std::fs::File::create(out_path.join("archs.rs")).expect("Failed to create a file");
    write!(arch_file, "{}", archs).expect("Failed to write to a file");
}
