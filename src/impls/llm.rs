mod claude;
mod openai;

use self::openai::{ChatCompletionBackend, ResponsesBackend};
use crate::{
    error::LlmError,
    model::{
        config::{AppConfigLlm, AppConfigLlmBackend, AppConfigLlmOpenaiApi},
        conversation::ASSISTANT_RESPONSE_SCHEMA,
        schema::{DescribedSchema, DescribedSchemaType},
    },
    specs::llm::Llm,
};

use std::{collections::HashMap, sync::LazyLock};

use async_openai::{error::OpenAIError, types::ResponseFormatJsonSchema};
use reqwest::Error as ReqwestError;
use serde_json::{Value as JsonValue, json};

pub async fn create_llm(config: &AppConfigLlm) -> Result<Box<dyn Llm + 'static>, LlmError> {
    match config.backend {
        AppConfigLlmBackend::Openai => match config.openai.api {
            AppConfigLlmOpenaiApi::ChatCompletion => Ok(Box::new(ChatCompletionBackend::new(&config.openai).await?)),
            AppConfigLlmOpenaiApi::Resnposes => Ok(Box::new(ResponsesBackend::new(&config.openai).await?)),
        },
    }
}

impl From<OpenAIError> for LlmError {
    fn from(value: OpenAIError) -> Self {
        LlmError::Backend(value.into())
    }
}

impl From<ReqwestError> for LlmError {
    fn from(value: ReqwestError) -> Self {
        LlmError::Communication(value.into())
    }
}

fn convert_json_schema(schema: &DescribedSchema) -> JsonValue {
    match &schema.field_type {
        DescribedSchemaType::Integer => json!({
            "type": "integer",
            "description": schema.description,
        }),
        DescribedSchemaType::Float => json!({
            "type": "float",
            "description": schema.description,
        }),
        DescribedSchemaType::Boolean => json!({
            "type": "boolean",
            "description": schema.description,
        }),
        DescribedSchemaType::String => json!({
            "type": "string",
            "description": schema.description,
        }),
        DescribedSchemaType::Object(fields) => {
            let properties: HashMap<_, _> = fields
                .iter()
                .map(|f| (f.name.clone(), convert_json_schema(f)))
                .collect();
            let keys: Vec<_> = properties.keys().cloned().collect();
            json!({
                "type": "object",
                "properties": properties,
                "required": keys,
                "additionalProperties": false,
            })
        }
    }
}

static RESPONSE_JSON_SCHEMA: LazyLock<ResponseFormatJsonSchema> = LazyLock::new(|| ResponseFormatJsonSchema {
    name: "response".into(),
    description: Some("response from assistant".into()),
    schema: Some(convert_json_schema(&ASSISTANT_RESPONSE_SCHEMA)),
    strict: Some(true),
});
