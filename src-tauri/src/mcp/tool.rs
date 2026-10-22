//! A tool: a name, a description for the agent, an input schema, and a handler.

use std::future::Future;
use std::pin::Pin;

use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::ctx::Ctx;
use crate::error::{AppError, AppResult};

/// What running a tool returns: its result as JSON, once it's done.
pub type Running = Pin<Box<dyn Future<Output = AppResult<Value>> + Send>>;

type Handler = Box<dyn Fn(Ctx, Value) -> Running + Send + Sync>;

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
    pub call: Handler,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// The input schema generated from a tool's argument type `A`.
fn schema<A: JsonSchema>() -> Value {
    let mut schema = serde_json::to_value(schemars::schema_for!(A)).unwrap_or(Value::Null);
    if let Some(obj) = schema.as_object_mut() {
        obj.remove("$schema");
        obj.remove("title");
    }
    schema
}

/// Builds a tool whose input schema is generated from its argument type `A`.
pub fn tool<A, R>(
    name: &'static str,
    description: &'static str,
    f: fn(&Ctx, A) -> AppResult<R>,
) -> Tool
where
    A: DeserializeOwned + JsonSchema + 'static,
    R: Serialize + 'static,
{
    Tool {
        name,
        description,
        schema: schema::<A>(),
        call: Box::new(move |ctx, args| {
            let done = serde_json::from_value::<A>(args)
                .map_err(AppError::from)
                .and_then(|a| Ok(serde_json::to_value(f(&ctx, a)?)?));
            Box::pin(std::future::ready(done))
        }),
    }
}

/// [`tool`] for a handler that waits on the database.
pub fn async_tool<A, R, F, Fut>(name: &'static str, description: &'static str, f: F) -> Tool
where
    A: DeserializeOwned + JsonSchema + Send + 'static,
    R: Serialize + 'static,
    F: Fn(Ctx, A) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = AppResult<R>> + Send + 'static,
{
    Tool {
        name,
        description,
        schema: schema::<A>(),
        call: Box::new(move |ctx, args| {
            let running = serde_json::from_value::<A>(args).map(|a| f(ctx, a));
            Box::pin(async move { Ok(serde_json::to_value(running?.await?)?) })
        }),
    }
}

/// For tools that take no arguments.
#[derive(Debug, Default, serde::Deserialize, JsonSchema)]
pub struct NoArgs {}
