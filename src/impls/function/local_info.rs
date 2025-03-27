use crate::{
    error::FunctionError,
    model::schema::DescribedSchema,
    specs::function::simple::{SimpleFunction, SimpleFunctionDescriptor},
};

use futures::{FutureExt, future::BoxFuture};
use serde_json::{Value, json};
use time::{
    OffsetDateTime,
    error::{Format, IndeterminateOffset},
    format_description::well_known::Rfc3339,
};

#[derive(Debug)]
pub struct LocalInfo {
    started_at: OffsetDateTime,
}

impl SimpleFunction for LocalInfo {
    fn get_descriptor(&self) -> SimpleFunctionDescriptor {
        SimpleFunctionDescriptor {
            name: "self_info".to_string(),
            description: "この bot が動作している環境の情報(現在時刻、uptime など)を取得する。".to_string(),
            parameters: DescribedSchema::object("parameters", "引数", vec![]),
        }
    }

    fn call<'a>(&'a self, _id: &str, _params: Value) -> BoxFuture<'a, Result<Value, FunctionError>> {
        async { self.get_info() }.boxed()
    }
}

impl LocalInfo {
    pub fn new() -> Result<LocalInfo, FunctionError> {
        Ok(LocalInfo {
            started_at: OffsetDateTime::now_local()?,
        })
    }

    fn get_info(&self) -> Result<Value, FunctionError> {
        let now = OffsetDateTime::now_local()?;
        Ok(json!({
            "time_now": now.format(&Rfc3339)?,
            "bot_started_at": self.started_at.format(&Rfc3339)?,
        }))
    }
}

impl From<IndeterminateOffset> for FunctionError {
    fn from(value: IndeterminateOffset) -> Self {
        FunctionError::External(value.into())
    }
}

impl From<Format> for FunctionError {
    fn from(value: Format) -> Self {
        FunctionError::External(value.into())
    }
}
