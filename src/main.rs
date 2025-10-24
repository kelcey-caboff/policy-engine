use std::error::Error;
use std::fs;
use std::path::Path;
use serde_json::Value;
use clap::Parser;

use policy_engine::{PolicyEngine};

#[derive(Parser, Debug)]
struct Args {
    /// Location of the policies folder
    #[arg(short, long)]
    policies: String,

    /// Metadata file to be evaluated
    #[arg(short, long)]
    metadata: String,

    /// Output JSON
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut engine = PolicyEngine::new();
    
    let args = Args::parse();

    let policies_path = Path::new(&args.policies);

    println!("Loading policies from {:?}...", policies_path.canonicalize()?);

    for entry in fs::read_dir(policies_path)? {
        let entry = entry?;
        let path = entry.path();
        
        // Ensure we only read .json files
        if path.is_file() && path.extension().map_or(false, |s| s == "json") {
            let filename = path.file_name().unwrap().to_string_lossy();
            println!("  - Loading rules from {:?}", filename);

            let json_data = match fs::read_to_string(&path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("    - Error: Failed to read policy file {:?}: {}", filename, e);
                    continue;
                }
            };

            if let Err(e) = engine.add_policies_from_json(&json_data) {
                // Print error to stderr and skip this file
                eprintln!("    - Error: Failed to parse policy file {:?}: {}", filename, e);
                continue
            }
        }
    }

    let meta_file = Path::new(&args.metadata);
    let filename = meta_file.file_name().unwrap().to_string_lossy();
    println!("Validating {}", &args.metadata);
    let metadata_file_str = match fs::read_to_string(meta_file) {
        Ok(data) => data,
        Err(e) => {
            panic!("Error: Failed to read metadata file {:?}: {}", filename, e);
        }
    };

    let metadata_file_parsed: Value = match serde_json::from_str(&metadata_file_str) {
        Ok(data) => data,
        Err(e) => {
            panic!("Error: Failed to parse metadata file {:?}: {}", filename, e);
        }
    };
    
    let metadata_file_result = engine.validate(&metadata_file_parsed);
    
    match &args.output {
        Some(output_file) => {
            let json_report = metadata_file_result.to_json()?;
            fs::write(&output_file, json_report)?;
            
            println!("Wrote JSON report to {:?}", output_file);
        },
        _ => println!("{}", metadata_file_result),
    }

    Ok(())
}