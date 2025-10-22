use std::error::Error;
use std::fs;
use std::path::Path;
use serde_json::Value;

use policy_engine::{PolicyEngine};

fn main() -> Result<(), Box<dyn Error>> {

    let mut engine = PolicyEngine::new();
    let policies_path = Path::new("policies");

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

    println!("\n--- Validating 'project_pass.json' (Should PASS) ---");
    let metadata_pass_str = fs::read_to_string("metadata/project_pass.json")?;
    let metadata_pass: Value = serde_json::from_str(&metadata_pass_str)?;
    
    let result_pass = engine.validate(&metadata_pass);
    println!("{}", result_pass);

    println!("--- Validating 'project_fail.json' (Should FAIL) ---");
    let metadata_fail_str = fs::read_to_string("metadata/project_fail.json")?;
    let metadata_fail: Value = serde_json::from_str(&metadata_fail_str)?;
    
    let result_fail = engine.validate(&metadata_fail);
    println!("{}", result_fail);
    println!("JSON Report:\n{}", result_fail.to_json()?);

    Ok(())
}