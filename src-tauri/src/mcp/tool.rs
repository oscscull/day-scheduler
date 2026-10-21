//! A tool: a name, a description for the agent, an input schema, and a handler.

use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::ctx::Ctx;
use crate::error::AppResult;

type Handler = Box<dyn Fn(&Ctx, Value) -> AppResult<Value>>;

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
    let mut schema = serde_json::to_value(schemars::schema_for!(A)).unwrap_or(Value::Null);
    if let Some(obj) = schema.as_object_mut() {
        obj.remove("$schema");
        obj.remove("title");
    }
    Tool {
        name,
        description,
        schema,
        call: Box::new(move |ctx, args| {
            let parsed: A = serde_json::from_value(args)?;
            Ok(serde_json::to_value(f(ctx, parsed)?)?)
        }),
    }
}

/// For tools that take no arguments.
#[derive(Debug, Default, serde::Deserialize, JsonSchema)]
pub struct NoArgs {}
