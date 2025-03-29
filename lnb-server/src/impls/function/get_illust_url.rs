use crate::{
    error::FunctionError,
    model::{config::AppConfigToolGetIllustUrl, schema::DescribedSchema},
    specs::function::simple::{SimpleFunction, SimpleFunctionDescriptor, SimpleFunctionResponse},
};

use futures::{FutureExt, future::BoxFuture};
use rand::{rng, seq::IndexedRandom};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{SqlitePool, prelude::FromRow};

#[derive(Debug)]
pub struct GetIllustUrl {
    pool: SqlitePool,
}

impl SimpleFunction for GetIllustUrl {
    fn get_descriptor(&self) -> SimpleFunctionDescriptor {
        SimpleFunctionDescriptor {
            name: "get_skeb_url".to_string(),
            description: r#"
                この bot 自身をキャラクターとして描写したイラストの URL を取得する。
                自画像・自撮りを要求された場合もこれを利用する。
            "#
            .to_string(),
            parameters: DescribedSchema::object(
                "parameters",
                "引数",
                vec![DescribedSchema::integer("count", "要求したいイラストの URL の数")],
            ),
        }
    }

    fn call<'a>(&'a self, _id: &str, params: Value) -> BoxFuture<'a, Result<SimpleFunctionResponse, FunctionError>> {
        let count = params["count"].as_u64().unwrap_or(1) as usize;
        async move { self.get_illust_infos(count).await }.boxed()
    }
}

impl GetIllustUrl {
    pub async fn new(config: &AppConfigToolGetIllustUrl) -> Result<GetIllustUrl, FunctionError> {
        let pool = SqlitePool::connect(&config.database_filepath).await?;
        Ok(GetIllustUrl { pool })
    }

    async fn get_illust_infos(&self, count: usize) -> Result<SimpleFunctionResponse, FunctionError> {
        let all_illusts: Vec<IllustInfo> = sqlx::query_as(r#"SELECT url, creator_name, comment FROM skeb_illusts;"#)
            .fetch_all(&self.pool)
            .await?;

        let limited_count = count.min(4).min(all_illusts.len());
        let selected_illusts: Vec<_> = all_illusts.choose_multiple(&mut rng(), limited_count).collect();

        Ok(SimpleFunctionResponse {
            result: json!({
                "illusts": selected_illusts
            }),
            ..Default::default()
        })
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct IllustInfo {
    pub url: String,
    pub creator_name: String,
    pub comment: String,
}
