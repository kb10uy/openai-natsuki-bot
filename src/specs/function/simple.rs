use crate::{error::FunctionError, model::schema::DescribedSchema};

use std::fmt::Debug;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleFunctionDescriptor {
    pub name: String,
    pub description: String,
    pub parameters: DescribedSchema,
}

pub trait SimpleFunction: Send + Sync + Debug {
    /// この `SimpleFunction` のディスクリプタを返す。
    fn get_descriptor(&self) -> SimpleFunctionDescriptor;

    /// Function を実行する。
    fn call<'a>(&'a self, id: &str, params: Value) -> BoxFuture<'a, Result<String, FunctionError>>;
}
