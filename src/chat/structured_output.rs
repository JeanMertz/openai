use schemars::{schema_for, transform::AddNullable, transform::Transform as _, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JsonSchemaStyle {
    OpenAI,
    Grok,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ChatCompletionResponseFormatJsonSchema {
    /// The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    /// A description of what the response format is for, used by the model to determine how to respond in the format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The schema for the response format, described as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
    /// Whether to enable strict schema adherence when generating the output.
    /// If set to true, the model will always follow the exact schema defined in the schema field.
    /// Only a subset of JSON Schema is supported when strict is true.
    /// To learn more, read the [Structured Outputs guide](https://platform.openai.com/docs/guides/structured-outputs).
    ///
    /// defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl ChatCompletionResponseFormatJsonSchema {
    pub fn new<T: JsonSchema>(strict: bool, json_style: JsonSchemaStyle) -> Self {
        let (schema, description) = generate_json_schema::<T>(json_style);
        ChatCompletionResponseFormatJsonSchema {
            name: T::schema_name().into_owned(),
            description,
            schema: Some(schema),
            strict: Some(strict),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct ToolCallFunctionDefinition {
    /// The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    /// A description of what the function does, used by the model to choose when and how to call the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The parameters the functions accepts, described as a JSON Schema object.
    /// See the [guide](https://platform.openai.com/docs/guides/function-calling) for examples,
    /// and the [JSON Schema reference](https://json-schema.org/understanding-json-schema/reference) for documentation about the format.
    /// Omitting `parameters` defines a function with an empty parameter list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    /// Whether to enable strict schema adherence when generating the function call.
    /// If set to true, the model will follow the exact schema defined in the `parameters` field.
    /// Only a subset of JSON Schema is supported when `strict` is `true`.
    /// Learn more about Structured Outputs in the [function calling guide](https://platform.openai.com/docs/api-reference/chat/docs/guides/function-calling).
    ///
    /// defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl ToolCallFunctionDefinition {
    /// Create a new ToolCallFunctionDefinition with the given strictness and JSON Schema style.
    ///
    /// Note: Grok tools does not support strict schema adherence, need to set `strict` to None.
    pub fn new<T: JsonSchema>(strict: Option<bool>) -> Self {
        let schema = schema_for!(T);
        let description = schema
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned);

        ToolCallFunctionDefinition {
            description,
            name: T::schema_name().into_owned(),
            parameters: Some(json!(schema)),
            strict,
        }
    }
}

/// Generate a JSON Schema with the given style.
///
/// IMPORTANT: Both OpenAI and Grok do not support the `format` and `minimum` JSON Schema attributes.
/// As a result, numeric type constraints (like `u8`, `i32`, etc) cannot be enforced - all integers
/// will be treated as `i64` and all floating point numbers as `f64`.
pub fn generate_json_schema<T: JsonSchema>(json_style: JsonSchemaStyle) -> (Value, Option<String>) {
    let mut settings = schemars::generate::SchemaSettings::default();
    settings.inline_subschemas = true;
    let mut generator = schemars::SchemaGenerator::new(settings);
    let mut schema = T::json_schema(&mut generator);

    if matches!(json_style, JsonSchemaStyle::Grok) {
        AddNullable::default().transform(&mut schema);
    }

    let description = schema
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);

    (schema.to_value(), description)
}
