use std::fs;
use policy_engine::generate_schema;

fn main() -> std::io::Result<()> {
    println!("Generating policy.schema.json...");

    let schema_json = generate_schema();

    fs::write("policy.schema.json", schema_json)?;
    println!("Successfully generated policy.schema.json!");
    
    Ok(())
}