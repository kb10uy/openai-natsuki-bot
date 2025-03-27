mod chat_completion;
mod responses;

pub use chat_completion::ChatCompletionBackend;
pub use responses::ResponsesBackend;

use crate::{
    USER_AGENT,
    error::LlmError,
    model::{
        config::AppConfigLlmOpenai,
        conversation::ASSISTANT_RESPONSE_SCHEMA,
        schema::{DescribedSchema, DescribedSchemaType},
    },
};

use std::{collections::HashMap, sync::LazyLock};

use async_openai::{Client, config::OpenAIConfig, types::ResponseFormatJsonSchema};
use serde_json::{Value as JsonValue, json};

static RESPONSE_JSON_SCHEMA: LazyLock<ResponseFormatJsonSchema> = LazyLock::new(|| ResponseFormatJsonSchema {
    name: "response".into(),
    description: Some("response from assistant".into()),
    schema: Some(convert_schema(&ASSISTANT_RESPONSE_SCHEMA)),
    strict: Some(true),
});

async fn create_openai_client(openai_config: &AppConfigLlmOpenai) -> Result<Client<OpenAIConfig>, LlmError> {
    let config = OpenAIConfig::new()
        .with_api_key(&openai_config.token)
        .with_api_base(&openai_config.endpoint);
    let http_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build()?;

    let client = Client::with_config(config).with_http_client(http_client);
    Ok(client)
}

fn convert_schema(schema: &DescribedSchema) -> JsonValue {
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
            let properties: HashMap<_, _> = fields.iter().map(|f| (f.name.clone(), convert_schema(f))).collect();
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
