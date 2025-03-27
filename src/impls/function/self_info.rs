use crate::{
    error::FunctionError,
    model::schema::DescribedSchema,
    specs::function::simple::{SimpleFunction, SimpleFunctionDescriptor},
};

use futures::{FutureExt, future::BoxFuture};
use serde_json::{Value, json};

#[derive(Debug)]
pub struct SelfInfo {}

impl SimpleFunction for SelfInfo {
    fn get_descriptor(&self) -> SimpleFunctionDescriptor {
        SimpleFunctionDescriptor {
            name: "self_info".to_string(),
            description: "この bot 自身の情報を取得する。".to_string(),
            parameters: DescribedSchema::object("parameters", "引数", vec![]),
        }
    }

    fn call<'a>(&'a self, _id: &str, _params: Value) -> BoxFuture<'a, Result<Value, FunctionError>> {
        async { self.get_info() }.boxed()
    }
}

impl SelfInfo {
    fn get_info(&self) -> Result<Value, FunctionError> {
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "commit": env!("GIT_COMMIT_HASH"),
        }))
    }
}
