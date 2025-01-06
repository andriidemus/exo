use anyhow::Result;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::prelude::SessionContext;

pub struct LocalSession {
    ctx: SessionContext,
}

pub trait Session {
    async fn sql(&self, expr: &str) -> Result<Vec<RecordBatch>>;
}

impl Default for LocalSession {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSession {
    pub fn new() -> Self {
        Self {
            ctx: SessionContext::new(),
        }
    }
}

impl Session for LocalSession {
    async fn sql(&self, expr: &str) -> Result<Vec<RecordBatch>> {
        let result = self.ctx.sql(expr).await?.collect().await?;
        Ok(result)
    }
}
