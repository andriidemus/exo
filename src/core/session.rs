use anyhow::Result;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::prelude::SessionContext;
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
        Self {
            ctx: SessionContext::new(),
        }
    }
}

impl DataFusionSession for LocalDataFusionSession {
    async fn sql(&self, expr: &str) -> Result<Vec<RecordBatch>> {
        let batches = self.ctx.sql(expr).await?.collect().await?;
        Ok(batches)
    }
}
