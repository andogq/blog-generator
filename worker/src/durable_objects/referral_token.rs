use serde_json::json;
use worker::*;

#[durable_object]
pub struct Counter {
    state: State,
    env: Env,
    value: i32,
    initialised: bool,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            value: 0,
            initialised: false,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        if !self.initialised {
            self.initialised = true;
            self.value = self.state.storage().get("value").await.unwrap_or_default();
        }

        match req.path().as_str() {
            "/increment" => {
                self.value += 1;
            }
            "/decrement" => {
                self.value -= 1;
            }
            _ => (),
        }

        self.state.storage().put("value", self.value).await?;

        Response::from_json(&json!({
            "value": self.value
        }))
    }
}
