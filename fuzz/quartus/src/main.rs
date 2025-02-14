use std::{fmt::Write, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};

mod quartus;

#[derive(Deserialize, Serialize)]
struct Part {
    name: String,
    family: String,
    device: String,
    package: String,
    pin_count: u32,
}

impl Part {
    pub fn new(name: &str, family: &str, device: &str, package: &str, pin_count: &str) -> Self {
        Self {
            name: String::from(name),
            family: String::from(family),
            device: String::from(device),
            package: String::from(package),
            pin_count: u32::from_str(pin_count).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Database {
    parts: Option<Vec<Part>>,
}

impl Database {
    pub fn new() -> Self {
        Self { parts: None }
    }

    pub fn parts(q: &quartus::Quartus) -> Vec<Part> {
        let cmd = q
            .run_tcl("quartus_cdb.exe", "puts [get_part_list]")
            .unwrap();

        assert!(cmd.len() > 0);

        let mut parts = Vec::new();

        let part_info_re = Regex::new(r"{.*} .* .* .*").unwrap();

        for line in cmd {
            let mut cmd = String::new();
            for part in line.split_whitespace() {
                writeln!(
                    cmd,
                    "puts [ get_part_info -family -device -package -pin_count {} ]",
                    part
                )
                .unwrap();
            }
            let data = q.run_tcl("quartus_cdb.exe", &cmd).unwrap();
            for (metadata, part_name) in data.iter().zip(line.split_whitespace()) {
                let captures = part_info_re.captures(metadata).unwrap();
                let (family, device, package, pin_count) =
                    (&captures[0], &captures[1], &captures[2], &captures[3]);
                parts.push(Part::new(part_name, family, device, package, pin_count));
            }
        }

        parts
    }
}

fn build(q: &quartus::Quartus, filenames: &[&str], top_level: &str) {
    let mut cmd = String::new();
    writeln!(cmd, "project_new -overwrite {}", top_level);
    for filename in filenames {
        writeln!(
            cmd,
            "set_global_assignment -name VERILOG_FILE \"{}\"",
            filename
        );
    }
    writeln!(
        cmd,
        "set_global_assignment -name TOP_LEVEL_ENTITY {}",
        top_level
    );
    writeln!(cmd, "set_global_assignment -name NUM_PARALLEL_PROCESSORS 1");
    writeln!(cmd, "export_assignments");
    writeln!(cmd, "project_close");
    writeln!(cmd, "project_open {}", top_level);
    writeln!(cmd, "load_package flow");
    writeln!(cmd, "execute_flow -compile");
    writeln!(cmd, "execute_flow -vqm_writer");

    let output = q.run_tcl("quartus_sh.exe", &cmd).unwrap();

    for line in output {
        println!("{}", line);
    }

    let output = q
        .run_arg("quartus_cdb.exe", &["--back_annotate=routing", top_level])
        .unwrap();

    for line in output {
        println!("{}", line);
    }
}

fn main() {
    let q = quartus::Quartus::new(&"/mnt/d/intelFPGA_lite/18.1/quartus/bin64/").unwrap();

    /*
    const DATABASE_NAME: &str = "mistral.json";

    let _db = Database::new();
    */
    build(
        &q,
        &[r"//wsl\$/Ubuntu-18.04/home/lofty/cyclonev/cyclone_re/crc32.v"],
        "cyclone_re",
    );
}
