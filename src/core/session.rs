use anyhow::Result;
use datafusion::prelude::SessionContext;
use serde_json::json;
use std::future::Future;

pub struct LocalDataFusionSession {
    ctx: SessionContext,
}

pub trait DataFusionSession {
    fn sql(&self, expr: &str) -> impl Future<Output = Result<serde_json::Value>> + Send;
}

impl Default for LocalDataFusionSession {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalDataFusionSession {
    pub fn new() -> Self {
        Self {
            ctx: SessionContext::new(),
        }
    }
}

impl DataFusionSession for LocalDataFusionSession {
    async fn sql(&self, expr: &str) -> Result<serde_json::Value> {
        let _result = self.ctx.sql(expr).await?.collect();
        // TODO: convert record batch to serde json
        Ok(json!("test1"))
    }
}
