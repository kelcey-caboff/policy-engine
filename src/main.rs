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
            println!("  - Loading rules from {:?}", path.file_name().unwrap());
            let json_data = fs::read_to_string(&path)?;
            engine.add_policies_from_json(&json_data)
                .map_err(|e| format!("Error parsing {}: {}", path.display(), e))?;
        }
    }

    let meta_file = Path::new(&args.metadata);
    println!("--- Validating {} ---", &args.metadata);
    let metadata_file_str = fs::read_to_string(meta_file)?;
    let metadata_file_parsed: Value = serde_json::from_str(&metadata_file_str)?;
    
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