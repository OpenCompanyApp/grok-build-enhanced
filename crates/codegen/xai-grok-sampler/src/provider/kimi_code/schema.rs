//! Kimi-specific JSON Schema compatibility for function tools.
//!
//! Some MCP servers emit valid schemas that omit nested `type` fields or use
//! local definition references. Kimi's tool validator is stricter than a
//! general JSON Schema validator, so this module repairs only the provider
//! wire copy. The original tool definitions and every other provider remain
//! unchanged.

use std::collections::HashSet;

use serde_json::{Map, Value};
use xai_grok_sampling_types::{Result, SamplingError};

const MAX_SCHEMA_DEPTH: usize = 128;
const MAX_SCHEMA_NODES: usize = 100_000;

#[derive(Clone, Copy)]
enum SlotKind {
    Single,
    Array,
    Map,
    SchemaOrArray,
}

#[derive(Clone, Copy)]
struct ChildSlot {
    key: &'static str,
    kind: SlotKind,
    parent_type: Option<JsonType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum JsonType {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Null,
}

impl JsonType {
    fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Object => "object",
            Self::Array => "array",
            Self::Null => "null",
        }
    }
}

const CHILD_SLOTS: &[ChildSlot] = &[
    ChildSlot {
        key: "$defs",
        kind: SlotKind::Map,
        parent_type: None,
    },
    ChildSlot {
        key: "definitions",
        kind: SlotKind::Map,
        parent_type: None,
    },
    ChildSlot {
        key: "dependencies",
        kind: SlotKind::Map,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "dependentSchemas",
        kind: SlotKind::Map,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "patternProperties",
        kind: SlotKind::Map,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "properties",
        kind: SlotKind::Map,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "additionalItems",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Array),
    },
    ChildSlot {
        key: "additionalProperties",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "contains",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Array),
    },
    ChildSlot {
        key: "contentSchema",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::String),
    },
    ChildSlot {
        key: "else",
        kind: SlotKind::Single,
        parent_type: None,
    },
    ChildSlot {
        key: "if",
        kind: SlotKind::Single,
        parent_type: None,
    },
    ChildSlot {
        key: "not",
        kind: SlotKind::Single,
        parent_type: None,
    },
    ChildSlot {
        key: "propertyNames",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "then",
        kind: SlotKind::Single,
        parent_type: None,
    },
    ChildSlot {
        key: "unevaluatedItems",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Array),
    },
    ChildSlot {
        key: "unevaluatedProperties",
        kind: SlotKind::Single,
        parent_type: Some(JsonType::Object),
    },
    ChildSlot {
        key: "allOf",
        kind: SlotKind::Array,
        parent_type: None,
    },
    ChildSlot {
        key: "anyOf",
        kind: SlotKind::Array,
        parent_type: None,
    },
    ChildSlot {
        key: "oneOf",
        kind: SlotKind::Array,
        parent_type: None,
    },
    ChildSlot {
        key: "prefixItems",
        kind: SlotKind::Array,
        parent_type: Some(JsonType::Array),
    },
    ChildSlot {
        key: "items",
        kind: SlotKind::SchemaOrArray,
        parent_type: Some(JsonType::Array),
    },
];

const TYPE_INFERENCE_SKIP_KEYS: &[&str] = &[
    "$ref", "allOf", "anyOf", "else", "if", "not", "oneOf", "then",
];
const OBJECT_STRUCTURE_KEYS: &[&str] = &[
    "dependencies",
    "dependentSchemas",
    "patternProperties",
    "properties",
    "additionalProperties",
    "propertyNames",
    "unevaluatedProperties",
    "dependentRequired",
    "maxProperties",
    "minProperties",
    "required",
];
const ARRAY_STRUCTURE_KEYS: &[&str] = &[
    "additionalItems",
    "contains",
    "unevaluatedItems",
    "prefixItems",
    "items",
    "maxContains",
    "maxItems",
    "minContains",
    "minItems",
    "uniqueItems",
];
const STRING_STRUCTURE_KEYS: &[&str] = &[
    "contentSchema",
    "contentEncoding",
    "contentMediaType",
    "format",
    "maxLength",
    "minLength",
    "pattern",
];
const NUMERIC_STRUCTURE_KEYS: &[&str] = &[
    "exclusiveMaximum",
    "exclusiveMinimum",
    "maximum",
    "minimum",
    "multipleOf",
];

#[derive(Default)]
struct WalkBudget {
    nodes: usize,
}

impl WalkBudget {
    fn visit(&mut self, depth: usize) -> Result<()> {
        self.nodes = self.nodes.saturating_add(1);
        if depth > MAX_SCHEMA_DEPTH || self.nodes > MAX_SCHEMA_NODES {
            return Err(schema_error(
                "Kimi Code tool schema exceeds compatibility-normalizer limits",
            ));
        }
        Ok(())
    }
}

pub(super) fn normalize_tool_schemas(body: &mut Value, backend: &'static str) -> Result<()> {
    let Some(tools) = body.get_mut("tools") else {
        return Ok(());
    };
    let tools = tools
        .as_array_mut()
        .ok_or_else(|| schema_error("Kimi Code tool declarations must be a JSON array"))?;

    for tool in tools {
        let schema = match backend {
            "chat_completions" => tool.pointer_mut("/function/parameters"),
            "messages" => tool.get_mut("input_schema"),
            _ => None,
        }
        .ok_or_else(|| schema_error("Kimi Code function tool is missing its input schema"))?;
        *schema = normalize_tool_schema(schema)?;
    }
    Ok(())
}

fn normalize_tool_schema(schema: &Value) -> Result<Value> {
    if !schema.is_object() {
        return Err(schema_error(
            "Kimi Code function tool input schema must be an object",
        ));
    }

    let mut visited_refs = HashSet::new();
    let mut budget = WalkBudget::default();
    let mut normalized = dereference_node(schema, schema, &mut visited_refs, 0, &mut budget)?;

    if !normalized.is_object() {
        return Err(schema_error(
            "Kimi Code function tool input schema must normalize to an object",
        ));
    }
    if !has_definition_ref(&normalized, "$defs", 0, &mut WalkBudget::default())? {
        normalized
            .as_object_mut()
            .expect("normalized schema checked as object")
            .remove("$defs");
    }
    if !has_definition_ref(&normalized, "definitions", 0, &mut WalkBudget::default())? {
        normalized
            .as_object_mut()
            .expect("normalized schema checked as object")
            .remove("definitions");
    }

    // The root is a container. Only schemas in known child-schema positions
    // receive Kimi's missing-type compatibility repair.
    visit_child_schemas(
        normalized
            .as_object_mut()
            .expect("normalized schema checked as object"),
        0,
        &mut WalkBudget::default(),
    )?;
    Ok(normalized)
}

fn dereference_node(
    node: &Value,
    root: &Value,
    visited_refs: &mut HashSet<String>,
    depth: usize,
    budget: &mut WalkBudget,
) -> Result<Value> {
    budget.visit(depth)?;
    match node {
        Value::Array(items) => items
            .iter()
            .map(|item| dereference_node(item, root, visited_refs, depth + 1, budget))
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str)
                && is_local_reference(reference)
            {
                if visited_refs.contains(reference) {
                    return Ok(Value::Object(object.clone()));
                }
                if let Some(target) = resolve_local_reference(root, reference) {
                    visited_refs.insert(reference.to_owned());
                    let resolved = dereference_node(target, root, visited_refs, depth + 1, budget)?;
                    visited_refs.remove(reference);
                    if let Value::Object(mut merged) = resolved {
                        for (key, value) in object {
                            if key == "$ref" {
                                continue;
                            }
                            merged.insert(
                                key.clone(),
                                dereference_node(value, root, visited_refs, depth + 1, budget)?,
                            );
                        }
                        return Ok(Value::Object(merged));
                    }
                    return Ok(resolved);
                }
            }

            let mut resolved = Map::new();
            for (key, value) in object {
                resolved.insert(
                    key.clone(),
                    dereference_node(value, root, visited_refs, depth + 1, budget)?,
                );
            }
            Ok(Value::Object(resolved))
        }
        scalar => Ok(scalar.clone()),
    }
}

fn is_local_reference(reference: &str) -> bool {
    reference == "#" || reference.starts_with("#/")
}

fn resolve_local_reference<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    if reference == "#" {
        return Some(root);
    }
    let mut current = root;
    for raw_part in reference.strip_prefix("#/")?.split('/') {
        let part = raw_part.replace("~1", "/").replace("~0", "~");
        current = match current {
            Value::Object(object) => object.get(&part)?,
            Value::Array(items) => {
                if part.len() > 1 && part.starts_with('0') {
                    return None;
                }
                items.get(part.parse::<usize>().ok()?)?
            }
            _ => return None,
        };
    }
    Some(current)
}

fn has_definition_ref(
    node: &Value,
    bucket: &str,
    depth: usize,
    budget: &mut WalkBudget,
) -> Result<bool> {
    budget.visit(depth)?;
    match node {
        Value::Array(items) => {
            for item in items {
                if has_definition_ref(item, bucket, depth + 1, budget)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Object(object) => {
            if object
                .get("$ref")
                .and_then(Value::as_str)
                .is_some_and(|reference| reference.starts_with(&format!("#/{bucket}/")))
            {
                return Ok(true);
            }
            for (key, value) in object {
                if key != bucket && has_definition_ref(value, bucket, depth + 1, budget)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn visit_child_schemas(
    object: &mut Map<String, Value>,
    depth: usize,
    budget: &mut WalkBudget,
) -> Result<()> {
    budget.visit(depth)?;
    for slot in CHILD_SLOTS {
        let Some(value) = object.get_mut(slot.key) else {
            continue;
        };
        match slot.kind {
            SlotKind::Single => {
                if value.is_object() {
                    normalize_property(value, depth + 1, budget)?;
                }
            }
            SlotKind::Array => {
                if let Some(items) = value.as_array_mut() {
                    for item in items {
                        if item.is_object() {
                            normalize_property(item, depth + 1, budget)?;
                        }
                    }
                }
            }
            SlotKind::Map => {
                if let Some(items) = value.as_object_mut() {
                    for item in items.values_mut() {
                        if item.is_object() {
                            normalize_property(item, depth + 1, budget)?;
                        }
                    }
                }
            }
            SlotKind::SchemaOrArray => {
                if value.is_object() {
                    normalize_property(value, depth + 1, budget)?;
                } else if let Some(items) = value.as_array_mut() {
                    for item in items {
                        if item.is_object() {
                            normalize_property(item, depth + 1, budget)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn normalize_property(value: &mut Value, depth: usize, budget: &mut WalkBudget) -> Result<()> {
    budget.visit(depth)?;
    let object = value
        .as_object_mut()
        .expect("normalize_property receives only objects");
    let skips_completion = contains_any(object, TYPE_INFERENCE_SKIP_KEYS);

    if !object.contains_key("type") && !skips_completion {
        let inferred = match object.get("enum").and_then(Value::as_array) {
            Some(values) if !values.is_empty() => infer_type_from_values(values)?,
            _ if object.contains_key("const") => {
                infer_type_from_values(&[object["const"].clone()])?
            }
            _ => infer_type_from_structure(object),
        };
        object.insert(
            "type".to_owned(),
            Value::String(inferred.as_str().to_owned()),
        );
    } else if !skips_completion && object.get("type").and_then(Value::as_str).is_some() {
        let inferred = match object.get("enum").and_then(Value::as_array) {
            Some(values) if !values.is_empty() => infer_type_from_values(values).ok(),
            _ if object.contains_key("const") => {
                infer_type_from_values(&[object["const"].clone()]).ok()
            }
            _ => None,
        };
        if let Some(inferred) = inferred
            && object.get("type").and_then(Value::as_str) != Some(inferred.as_str())
        {
            object.insert(
                "type".to_owned(),
                Value::String(inferred.as_str().to_owned()),
            );
            remove_irrelevant_structure(object, inferred);
        }
    }

    visit_child_schemas(object, depth + 1, budget)
}

fn infer_type_from_values(values: &[Value]) -> Result<JsonType> {
    let mut inferred = HashSet::new();
    for value in values {
        inferred.insert(match value {
            Value::Null => JsonType::Null,
            Value::Bool(_) => JsonType::Boolean,
            Value::Number(number) if number.is_i64() || number.is_u64() => JsonType::Integer,
            Value::Number(_) => JsonType::Number,
            Value::String(_) => JsonType::String,
            Value::Array(_) => JsonType::Array,
            Value::Object(_) => JsonType::Object,
        });
    }
    if inferred.contains(&JsonType::Number) {
        inferred.remove(&JsonType::Integer);
    }
    if inferred.len() == 1 {
        return Ok(*inferred.iter().next().expect("one inferred type"));
    }
    Err(schema_error(
        "Kimi Code tool schema contains mixed enum or const value types",
    ))
}

fn infer_type_from_structure(object: &Map<String, Value>) -> JsonType {
    let has_parent_slot = |parent_type| {
        CHILD_SLOTS
            .iter()
            .any(|slot| slot.parent_type == Some(parent_type) && object.contains_key(slot.key))
    };
    if contains_any(object, OBJECT_STRUCTURE_KEYS) || has_parent_slot(JsonType::Object) {
        JsonType::Object
    } else if contains_any(object, ARRAY_STRUCTURE_KEYS) || has_parent_slot(JsonType::Array) {
        JsonType::Array
    } else if contains_any(object, STRING_STRUCTURE_KEYS) || has_parent_slot(JsonType::String) {
        JsonType::String
    } else if contains_any(object, NUMERIC_STRUCTURE_KEYS) {
        JsonType::Number
    } else {
        JsonType::String
    }
}

fn remove_irrelevant_structure(object: &mut Map<String, Value>, inferred: JsonType) {
    if inferred != JsonType::Object {
        for key in OBJECT_STRUCTURE_KEYS {
            object.remove(*key);
        }
    }
    if inferred != JsonType::Array {
        for key in ARRAY_STRUCTURE_KEYS {
            object.remove(*key);
        }
    }
}

fn contains_any(object: &Map<String, Value>, keys: &[&str]) -> bool {
    keys.iter().any(|key| object.contains_key(*key))
}

fn schema_error(message: &'static str) -> SamplingError {
    SamplingError::InvalidConfiguration(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_refs_and_missing_types_are_normalized_without_changing_the_root() {
        let schema = serde_json::json!({
            "properties": {
                "mode": {"$ref": "#/$defs/Mode"},
                "filters": {
                    "properties": {
                        "path": {"pattern": "^src/"},
                        "limit": {"minimum": 1}
                    },
                    "required": ["path"]
                }
            },
            "$defs": {"Mode": {"enum": ["fast", "safe"]}}
        });

        let normalized = normalize_tool_schema(&schema).unwrap();

        assert!(normalized.get("type").is_none());
        assert!(normalized.get("$defs").is_none());
        assert_eq!(normalized["properties"]["mode"]["type"], "string");
        assert_eq!(normalized["properties"]["filters"]["type"], "object");
        assert_eq!(
            normalized["properties"]["filters"]["properties"]["path"]["type"],
            "string"
        );
        assert_eq!(
            normalized["properties"]["filters"]["properties"]["limit"]["type"],
            "number"
        );
        assert_eq!(schema["properties"]["mode"]["$ref"], "#/$defs/Mode");
    }

    #[test]
    fn contradictory_enum_type_is_repaired_and_object_keywords_are_removed() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "object",
                    "enum": ["move", "copy"],
                    "properties": {"rawValue": {"type": "string"}},
                    "required": ["rawValue"]
                }
            }
        });

        let normalized = normalize_tool_schema(&schema).unwrap();

        assert_eq!(normalized["properties"]["operation"]["type"], "string");
        assert!(
            normalized["properties"]["operation"]
                .get("properties")
                .is_none()
        );
        assert!(
            normalized["properties"]["operation"]
                .get("required")
                .is_none()
        );
    }

    #[test]
    fn mixed_untyped_enum_is_rejected_before_the_provider_request() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"mode": {"enum": ["fast", 1]}}
        });

        let error = normalize_tool_schema(&schema).unwrap_err().to_string();

        assert_eq!(
            error,
            "invalid client configuration: Kimi Code tool schema contains mixed enum or const value types"
        );
    }

    #[test]
    fn cyclic_definition_references_keep_the_definition_bucket_resolvable() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"node": {"$ref": "#/$defs/Node"}},
            "$defs": {
                "Node": {
                    "type": "object",
                    "properties": {"next": {"$ref": "#/$defs/Node"}}
                }
            }
        });

        let normalized = normalize_tool_schema(&schema).unwrap();

        assert!(normalized.get("$defs").is_some());
        assert!(
            serde_json::to_string(&normalized)
                .unwrap()
                .contains("#/$defs/Node")
        );
    }
}
