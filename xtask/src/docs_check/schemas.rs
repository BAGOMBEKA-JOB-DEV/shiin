//! Checks JSON Schemas, validated examples embedded in pages, and specification test vectors.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use jsonschema::{Draft, Resource, Validator};
use serde_json::Value;
use walkdir::WalkDir;

use super::{Page, Patterns, Report, relative};

const SCHEMA_DIRECTORY: &str = "spec/schemas";
const TEST_VECTOR_DIRECTORY: &str = "spec/test-vectors";
const TEST_VECTOR_SCHEMA: &str = "spec/schemas/test-vector.v1.schema.json";
const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

pub(crate) fn check(docs: &Path, pages: &[Page], patterns: &Patterns, report: &mut Report) {
    let schemas = load_schemas(docs, patterns, report);
    let validators = compile_validators(&schemas, report);
    check_examples(pages, patterns, &validators, report);
    check_test_vectors(docs, &validators, report);
}

/// Reads every schema file, keyed by its path relative to `docs/`.
fn load_schemas(docs: &Path, patterns: &Patterns, report: &mut Report) -> BTreeMap<String, Value> {
    let mut schemas = BTreeMap::new();
    for (rel, value) in read_json_files(docs, SCHEMA_DIRECTORY, report) {
        if value.get("$schema").and_then(Value::as_str) != Some(DRAFT_2020_12) {
            report.error(&rel, format!("`$schema` must be `{DRAFT_2020_12}`"));
        }
        let file_name = rel.rsplit('/').next().unwrap_or(&rel);
        match value.get("$id").and_then(Value::as_str).and_then(|id| patterns.schema_id.captures(id)) {
            None => report.error(&rel, "`$id` must look like `urn:shiin:schema:<name>:v<major>`"),
            Some(captures) => {
                let name = captures.get(1).map_or("", |m| m.as_str());
                let major = captures.get(2).map_or("", |m| m.as_str());
                let expected_file = format!("{name}.v{major}.schema.json");
                if file_name != expected_file {
                    report.error(&rel, format!("file name must be `{expected_file}` to match `$id`"));
                }
            }
        }
        if let Err(error) = jsonschema::meta::validate(&value) {
            report.error(&rel, format!("not a valid JSON Schema: {error}"));
        }
        schemas.insert(rel, value);
    }
    schemas
}

/// Builds a validator for each schema, with every schema registered so `$ref` by `$id` resolves.
fn compile_validators(schemas: &BTreeMap<String, Value>, report: &mut Report) -> BTreeMap<String, Validator> {
    let mut validators = BTreeMap::new();
    for (rel, schema) in schemas {
        let mut options = jsonschema::options().with_draft(Draft::Draft202012);
        for other in schemas.values() {
            if let Some(id) = other.get("$id").and_then(Value::as_str) {
                options = options.with_resource(id, Resource::from_contents(other.clone()));
            }
        }
        match options.build(schema) {
            Ok(validator) => {
                validators.insert(rel.clone(), validator);
            }
            Err(error) => report.error(rel, format!("schema does not compile: {error}")),
        }
    }
    validators
}

/// Validates JSON code blocks that follow a `validate` or `validate-fail` marker.
fn check_examples(
    pages: &[Page],
    patterns: &Patterns,
    validators: &BTreeMap<String, Validator>,
    report: &mut Report,
) {
    for page in pages {
        let lines: Vec<&str> = page.text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let Some(captures) = patterns.validate_marker.captures(line) else {
                continue;
            };
            let location = format!("{}:{}", page.rel, index + 1);
            let should_pass = captures.get(1).is_some_and(|m| m.as_str() == "validate");
            let schema_path = captures.get(2).map_or("", |m| m.as_str());

            if !lines.get(index + 1).is_some_and(|next| next.trim_start().starts_with("```json")) {
                report.error(&location, "a validation marker must be followed directly by a ```json code block");
                continue;
            }
            let body: Vec<&str> = lines
                .iter()
                .skip(index + 2)
                .take_while(|body_line| !body_line.trim_start().starts_with("```"))
                .copied()
                .collect();
            let instance = match serde_json::from_str::<Value>(&body.join("\n")) {
                Ok(instance) => instance,
                Err(error) => {
                    report.error(&location, format!("example is not valid JSON: {error}"));
                    continue;
                }
            };
            let Some(validator) = validators.get(schema_path) else {
                report.error(&location, format!("schema `{schema_path}` does not exist or failed to compile"));
                continue;
            };
            match (should_pass, first_violation(validator, &instance)) {
                (true, Some(violation)) => {
                    report.error(&location, format!("example does not match {schema_path}: {violation}"));
                }
                (false, None) => {
                    report.error(&location, format!("example marked validate-fail matches {schema_path}"));
                }
                _ => {}
            }
        }
    }
}

/// Validates every test vector against the test-vector schema.
fn check_test_vectors(docs: &Path, validators: &BTreeMap<String, Validator>, report: &mut Report) {
    let vectors = read_json_files(docs, TEST_VECTOR_DIRECTORY, report);
    if vectors.is_empty() {
        return;
    }
    let Some(validator) = validators.get(TEST_VECTOR_SCHEMA) else {
        report.error(TEST_VECTOR_DIRECTORY, format!("test vectors exist but `{TEST_VECTOR_SCHEMA}` is missing"));
        return;
    };
    for (rel, vector) in vectors {
        if let Some(violation) = first_violation(validator, &vector) {
            report.error(&rel, format!("test vector does not match the schema: {violation}"));
        }
    }
}

fn first_violation(validator: &Validator, instance: &Value) -> Option<String> {
    validator
        .iter_errors(instance)
        .next()
        .map(|error| format!("{error} (at `{}`)", error.instance_path()))
}

/// Reads and parses every `.json` file under `docs/<directory>`, reporting unreadable files.
fn read_json_files(docs: &Path, directory: &str, report: &mut Report) -> Vec<(String, Value)> {
    let root = docs.join(directory);
    if !root.is_dir() {
        return Vec::new();
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&root).sort_by_file_name() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report.error(directory, error);
                continue;
            }
        };
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let rel = relative(docs, path);
        match fs::read_to_string(path).map_err(|error| error.to_string()).and_then(|text| {
            serde_json::from_str::<Value>(&text).map_err(|error| error.to_string())
        }) {
            Ok(value) => files.push((rel, value)),
            Err(error) => report.error(&rel, format!("cannot read JSON: {error}")),
        }
    }
    files
}
