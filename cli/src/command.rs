use anyhow::Result;
use async_trait::async_trait;

use crate::context::Context;

#[async_trait]
pub trait Command {
    async fn execute(&self, ctx: &Context) -> Result<()>;
}