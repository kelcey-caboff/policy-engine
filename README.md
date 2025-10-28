# Policy Engine

This crate provides a PolicyEngine for validating JSON metadata against a set of JSON-defined policy rules.

It can be used as a command-line tool or as a library in other Rust applications.

## Features

* Declarative Policies: Define complex validation rules in simple JSON files.
* CLI Tool: A ready-to-use CLI (policy_engine_cli) to validate metadata files against a folder of policies.
* JSON Schema Generation: Comes with a schema generator (schema_generator) to create a policy.schema.json file, enabling IDE autocompletion and validation for your policy files.
* Library (lib): A simple Rust API to embed the engine in your own applications.
* Rich Output: Provides human-readable pass/fail output and a machine-readable JSON report.
  
## How to Use (Command-Line)

The primary way to use this tool is via the policy_engine_cli binary.

### 1. Build the Tool

Build in release mode for best performance

```bash
cargo build --release
```

You can now find the executable at `./target/release/policy_engine_cli`

### 2. Run Validation
   
The CLI tool takes a folder of policy files and a single metadata file to validate.

Run using cargo:

```bash
cargo run --bin policy_engine_cli -- \
    --policies ./path/to/your_policies/ \
    --metadata ./path/to/your_metadata.json
```

Or run the compiled binary directly:
```bash
./target/release/policy_engine_cli \
    --policies ./my_policies \
    --metadata ./metadata.json
```

#### Arguments
`-p`, `--policies <PATH>`: Location of the policies folder. The tool will read all .json files in this directory.

`-m`, `--metadata <PATH>`: Metadata file to be evaluated.

`-o`, `--output <PATH>`: (Optional) Path to write a JSON report of the validation results.

### 3. Read the Output

#### Standard Output (Pass):

```
Loading policies from "/path/to/my_policies"...
  - Loading rules from "data_rules.json"
  - Loading rules from "llm_rules.json"
Validating ./metadata.json
Validation PASSED
```

#### Standard Output (Fail):

```
Loading policies from "/path/to/my_policies"...
  - Loading rules from "data_rules.json"
Validating ./metadata.json
Validation FAILED with 2 requirements:
  - 1: RULE "data_001" => If data is 'official', it must have an owner.
  - 2: RULE "llm_002" => If an LLM is used, it cannot train on internal data.
```

#### JSON Output (Fail):

If you use the `-o report.json` flag, a JSON file will be generated.

`report.json`:
```json
{
  "valid": false,
  "requirements": [
    {
      "control": "data_001",
      "required": "If data is 'official', it must have an owner."
    },
    {
      "control": "llm_002",
      "required": "If an LLM is used, it cannot train on internal data."
    }
  ]
}
```

## How to Write Policies

Policies are JSON files. The root of the file must be an object containing a `controls` array.

`my_policy.json`:
```json
{
  "controls": [
    {
      "id": "llm_policy_001",
      "description": "If you are using a LLM, you must have a named model owner.",
      "check": {
        "op": "if",
        "if": {
          "op": "some",
          "field": "/llm/model"
        },
        "then": {
          "op": "exists_and_not_empty",
          "field": "/llm/owner"
        }
      }
    }
    // ... more control objects ...
  ]
}
```

### The `check` object: Policy Operators

The `check` object defines the logic for the rule. All logic is based on the `op` field.

| Operator (op) | JSON Structure | What it Does |
| --- | --- | --- |
| `exists_and_not_empty` | `{ "op": "exists_and_not_empty", "field": "..." }` | Checks that the field exists, is a string, and is **not empty**. Fails for `null`, numbers, booleans, arrays, or `""`.|
| `equals` | `{ "op": "equals", "field": "...", "value": ... }` | Checks that the field exists and its value is **exactly equal** to the `value` provided. The `value` can be any JSON type (string, number, `true`, `false`, `null`).|
| `contains` | `{ "op": "contains", "field": "...", "value": ... }` | Checks that the field **exists**, is an **array**, and that array **contains** the provided value.|
`some` | `{ "op": "some", "field": "..." }` | Checks that the field exists and its value is not null. This will pass for `""`, `0`, `false`, or `[]`.|
| `if` | `{ "op": "if", "if": {...}, "then": {...}, "else": {...} }` | A standard If-Then-Else block.  1. The `if` check runs.  2. If `true`, the `then` check must pass.  3. If `false`, the `else` check must pass.  4. If `else` is omitted and `if` is `false`, the entire rule passes.|
|`allOf`|`{ "op": "allOf", "rules": [ ... ] }`|Logical **AND**. All checks in the `rules` array must pass.|
|`anyOf`|`{ "op": "anyOf", "rules": [ ... ] }`|Logical **OR**. At least one check in the `rules` array must pass.|
|`not`|`{ "op": "not", "rule": { ... } }`|Logical **NOT**. Inverts the result of the nested `rule`.|

## How to Write Metadata

The metadata file is a single JSON object. The policy engine accesses values within it using JSON Pointers (RFC 6901). A JSON Pointer is a string that starts with `/` and uses `/` to separate keys.

Example metadata.json:
```json
{
  "project": "gemini-nano",
  "data": {
    "sensitivity": "official",
    "handling": ["sensitive", "pii"]
  },
  "llm": {
    "model": "Gemini-1.5-Pro",
    "owner": "ai-platform-team"
  },
  "security_scans_complete": true
}
```

JSON Pointers for this metadata:

* `/project` -> `"gemini-nano"`
* `/data/sensitivity` -> `"official"`
* `/data/handling` -> `["sensitive", "pii"]`
* `/llm/owner` -> `"ai-platform-team"`
* `/security_scans_complete` -> `true`
  
## Generating the Schema (For Autocompletion)

To make writing policies easier, you can generate a `policy.schema.json` file. You can then reference this in your policy files to get autocompletion and validation in IDEs like VS Code.

### Run the Generator:

`cargo run --bin schema_generator`

This will create a `policy.schema.json` file in the root of your project. 

### Use the Schema in VS Code:

At the top of your `my_policy.json file`, add a `$schema key`:

```json
{
  "$schema": "./policy.schema.json",
  "controls": [ // <-- You will now get autocompletion here!
    {
      "id": "my_rule",
      "description": "My rule",
      "check": {
        "op": "equals", 
        "field": "/project",
        "value": "gemini-nano"
      }
    }
  ]
}
```

## Using as a Library

You can also use policy_engine as a library in your own Rust project.

### 1. Add to `Cargo.toml`

```toml
[dependencies]
policy_engine = { path = "/path/to/policy_engine" } # Or from crates.io
serde_json = "1.0.145"
```
### 2. Example Usage

```rust
use policy_engine::PolicyEngine;
use serde_json::json;
use std::fs;

fn main() {
    let mut engine = PolicyEngine::new();

    // Load policy files
    let policy_json = fs::read_to_string("./my_policies/llm_rules.json")
        .expect("Could not read policy file");
        
    engine.add_policies_from_json(&policy_json)
        .expect("Failed to parse policy JSON");

    // Create or load metadata
    let metadata = json!({
      "llm": {
        "model": "GPT-4",
        "owner": null // This will fail the "exists_and_not_empty" check
      }
    });

    // Validate
    let result = engine.validate(&metadata);

    // Print the human-readable result
    println!("{}", result);
    
    // Or get the JSON report
    let report_json = result.to_json().unwrap();
    println!("{}", report_json);
}
```