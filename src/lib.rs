use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use schemars::{schema_for, JsonSchema};

#[derive(Debug, Serialize)]
pub struct Requirements {
    pub control: String,
    pub required: String,
}

#[derive(Debug)]
pub enum Validation {
    True,
    False(Vec<Requirements>),
}

#[derive(Deserialize, JsonSchema)]
struct PolicyFile {
    controls: Vec<JsonControl>,
}

#[derive(Deserialize, JsonSchema)]
struct JsonControl {
    id: String,
    description: String,
    check: Check,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "op")]
enum Check {
    #[serde(rename = "exists_and_not_empty")]
    ExistsAndNotEmpty { field: String },

    #[serde(rename = "equals")]
    Equals { field: String, value: Value},

    #[serde(rename = "allOf")]
    AllOf { rules: Vec<Check> },

    #[serde(rename = "anyOf")]
    AnyOf { rules: Vec<Check> },

    #[serde(rename = "some")]
    Some { field: String},

    #[serde(rename = "not")]
    Not { rule: Box<Check> },

    #[serde(rename = "if")]
    If {
        #[serde(rename = "if")]
        if_cond: Box<Check>,
        #[serde(rename = "then")]
        then_cond: Box<Check>,
        #[serde(rename = "else", default, skip_serializing_if = "Option::is_none")]
        else_cond: Option<Box<Check>>,
    },

    #[serde(rename = "contains")]
    Contains { field: String, value: Value },
}

pub struct PolicyEngine {
    controls: Vec<JsonControl>
}


impl PolicyEngine {
    pub fn new() -> Self {
        PolicyEngine { controls: Vec::new() }
    }

    pub fn add_policies_from_json(&mut self, json_data: &str) -> Result<(), serde_json::Error> {
        let policy_file: PolicyFile = serde_json::from_str(json_data)?;
        self.controls.extend(policy_file.controls);
        Ok(())
    }

    pub fn validate(&self, metadata: &Value) -> Validation {
        let mut failed_reqs = Vec::new();

        for control in &self.controls {
            if !self.run_check(&control.check, metadata) {
                failed_reqs.push(Requirements {
                    control: control.id.clone(),
                    required: control.description.clone(),
                });
            }
        }

        if failed_reqs.is_empty() {
            Validation::True
        } else {
            Validation::False(failed_reqs)
        }
    }

    fn run_check(&self, check: &Check, metadata: &Value) -> bool {
        match check {
            Check::ExistsAndNotEmpty { field }  => {
                metadata.pointer(field)
                    .and_then(|v| v.as_str())
                    .map_or(false, |s| !s.is_empty())
            }
            Check::Equals {field, value} => {
                metadata.pointer(field).map_or(false, |v| v == value)
            }
            Check::AllOf { rules } => {
                rules.iter().all(|rule| self.run_check(rule, metadata))
            }
            Check::AnyOf { rules } => {
                rules.iter().any(|rule| self.run_check(rule, metadata))
            }
            Check::Not { rule } => {
                !self.run_check(rule, metadata)
            }
            Check::Some { field} => {
                metadata.pointer(field).map_or(false, |v| !v.is_null())
            }
            Check::If { if_cond, then_cond, else_cond} => {
                let if_result = self.run_check(if_cond, metadata);

                match else_cond {
                    None => !if_result || self.run_check(then_cond, metadata),

                    Some(else_rule) => {
                        if if_result {
                            self.run_check(then_cond, metadata)
                        } else {
                            self.run_check(else_rule, metadata)
                        }
                    }
                }
            }
            Check::Contains { field, value } => {
                metadata.pointer(field)
                    .and_then(|v| v.as_array())
                    .map_or(false, |arr| arr.contains(value))
            }
        }
    }
}

#[derive(Serialize)]
struct JsonValidation<'a> {
    valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requirements: &'a Vec<Requirements>,
}

impl Validation {
    pub fn to_json(&self) -> serde_json::Result<String> {
        let json_view = match self {
            Validation::True => JsonValidation {
                valid: true,
                requirements: &vec![],
            },
            Validation::False(r) => JsonValidation { 
                valid: false,
                requirements: r,
            }
        };

        serde_json::to_string_pretty(&json_view)
    }
}

impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Validation::True => writeln!(f, "Validation PASSED"),
            Validation::False(reqs) => {
                let plural = match reqs.len() {
                    1 => "requirement",
                    _ => "requirements",
                };
                writeln!(f, "Validation FAILED with {} {}:", reqs.len(), plural)?;
                for (i, r) in reqs.iter().enumerate() {
                    writeln!(f, "  - {}: RULE \"{}\" => {}", i+1, r.control, r.required)?;
                }
                Ok(())
            }
        }
    }
}

pub fn generate_schema() -> String {
    let schema = schema_for!(PolicyFile);
    serde_json::to_string_pretty(&schema).unwrap()
}