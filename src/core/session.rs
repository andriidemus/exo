use anyhow::Result;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::prelude::{SessionConfig, SessionContext};
use std::future::Future;

pub struct LocalDataFusionSession {
    ctx: SessionContext,
}

pub trait DataFusionSession {
    fn sql(&self, expr: &str) -> impl Future<Output = Result<Vec<RecordBatch>>> + Send;
}

impl Default for LocalDataFusionSession {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalDataFusionSession {
    pub fn new() -> Self {
        let conf = SessionConfig::new().with_information_schema(true);
        let ctx = SessionContext::new_with_config(conf);
        Self { ctx }
    }
}

impl DataFusionSession for LocalDataFusionSession {
    async fn sql(&self, expr: &str) -> Result<Vec<RecordBatch>> {
        let batches = self.ctx.sql(expr).await?.collect().await?;
        Ok(batches)
    }
}
