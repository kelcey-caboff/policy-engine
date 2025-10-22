use std::fs;
use std::path::PathBuf;
use serde_json::Value;
// Import our library's public API
use policy_engine::{PolicyEngine, Validation};

/// Helper function to load test files.
/// It builds a path relative to the test binary's location.
fn load_test_file(path: &str) -> String {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/test_cases");
    d.push(path);
    fs::read_to_string(d).expect(&format!("Failed to read test file: {}", path))
}

/// Helper function to run a single validation check
fn run_test(policy_file: &str, metadata_file: &str) -> Validation {
    let policy_json = load_test_file(policy_file);
    let metadata_json = load_test_file(metadata_file);
    
    let mut engine = PolicyEngine::new();
    engine.add_policies_from_json(&policy_json).expect("Failed to parse policy");
    
    let metadata: Value = serde_json::from_str(&metadata_json).expect("Failed to parse metadata");
    engine.validate(&metadata)
}

#[test]
fn test_op_exists_and_not_empty() {
    // Test pass case
    let pass_result = run_test("policies/policy_exists.json", "metadata/meta_exists_pass.json");
    match pass_result {
        Validation::True => {
            ()
        },
        _ => panic!("Test should have passed with Validation::True.")
    }

    // Test fail case
    let fail_result = run_test("policies/policy_exists.json", "metadata/meta_exists_fail.json");
    match fail_result {
        Validation::False(reqs) => {
            assert_eq!(reqs.len(), 1);
            assert_eq!(reqs[0].control, "exists_001");
        },
        _ => panic!("Test should have failed with Validation::False."),
    }
}

#[test]
fn test_op_equals() {
    let pass_result = run_test("policies/policy_equals.json", "metadata/meta_equals_pass.json");
    match pass_result {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    let fail_result = run_test("policies/policy_equals.json", "metadata/meta_equals_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "equals_001"),
        _ => panic!("Test should have failed.")
    }
}

#[test]
fn test_op_all_of() {
    let pass_result = run_test("policies/policy_allof.json", "metadata/meta_allof_pass.json");
    match pass_result {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    let fail_result = run_test("policies/policy_allof.json", "metadata/meta_allof_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "allof_001"),
        _ => panic!("Test should have failed."),
    }
}

#[test]
fn test_op_any_of() {
    let pass_result = run_test("policies/policy_anyof.json", "metadata/meta_anyof_pass.json");
    match pass_result {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    let fail_result = run_test("policies/policy_anyof.json", "metadata/meta_anyof_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "anyof_001"),
        _ => panic!("Test should have failed."),
    }
}

#[test]
fn test_op_not() {
    let pass_result = run_test("policies/policy_not.json", "metadata/meta_not_pass.json");
    match pass_result {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }
    let fail_result = run_test("policies/policy_not.json", "metadata/meta_not_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "not_001"),
        _ => panic!("Test should have failed."),
    }
}

#[test]
fn test_op_if_then() {
    // Pass case 1: (if true, then true)
    let pass_1 = run_test("policies/policy_if_then.json", "metadata/meta_if_then_pass_1.json");
    match pass_1 {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    // Pass case 2: (if false, then anything)
    let pass_2 = run_test("policies/policy_if_then.json", "metadata/meta_if_then_pass_2.json");
    match pass_2 {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    // Fail case: (if true, then false)
    let fail_result = run_test("policies/policy_if_then.json", "metadata/meta_if_then_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "if_then_001"),
        _ => panic!("Test should have failed."),
    }
}

#[test]
fn test_op_if_then_else() {
    // Pass case 1: (if true, then true)
    let pass_1 = run_test("policies/policy_if_then_else.json", "metadata/meta_if_then_else_pass_1.json");
    match pass_1 {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    // Pass case 2: (if false, else true)
    let pass_2 = run_test("policies/policy_if_then_else.json", "metadata/meta_if_then_else_pass_2.json");
    match pass_2 {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    // Fail case: (if true, then false)
    let fail_1 = run_test("policies/policy_if_then_else.json", "metadata/meta_if_then_else_fail_1.json");
    match fail_1 {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "if_else_001"),
        _ => panic!("Test should have failed"),
    }

    // Fail case: (if false, else false)
    let fail_2 = run_test("policies/policy_if_then_else.json", "metadata/meta_if_then_else_fail_2.json");
    match fail_2 {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "if_else_001"),
        _ => panic!("Test should have failed"),
    }
}

#[test]
fn test_op_some() {
    let pass_result = run_test("policies/policy_some.json", "metadata/meta_some_pass.json");
    match pass_result {
        Validation::True => (),
        _ => panic!("Test should have passed with Validation::True.")
    }

    let fail_result = run_test("policies/policy_some.json", "metadata/meta_some_fail.json");
    match fail_result {
        Validation::False(reqs) => assert_eq!(reqs[0].control, "some_001"),
        _ => panic!("Test should have failed"),
    }
}
