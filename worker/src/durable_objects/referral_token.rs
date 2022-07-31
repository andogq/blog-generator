use serde::Serialize;
use serde_json::json;
use worker::*;

#[derive(Clone, Serialize)]
struct Token {
    remaining_uses: u32,
    total_uses: u32,
    used_by: Vec<String>,
}

#[durable_object]
pub struct ReferralToken {
    state: State,
    env: Env,
    initialised: bool,
    token: Token,
}

#[durable_object]
impl DurableObject for ReferralToken {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            initialised: false,
            token: Token {
                remaining_uses: 0,
                total_uses: 0,
                used_by: Vec::new(),
            },
        }
    }

    async fn fetch(&mut self, mut req: Request) -> Result<Response> {
        if !self.initialised {
            self.initialised = true;

            self.token.remaining_uses = self
                .state
                .storage()
                .get("remaining_uses")
                .await
                .unwrap_or(0);
            self.token.total_uses = self.state.storage().get("total_uses").await.unwrap_or(0);
            self.token.used_by = self
                .state
                .storage()
                .get("used_by")
                .await
                .unwrap_or_else(|_| Vec::new());
        }

        let result = match (req.method(), req.path().as_str()) {
            (Method::Get, "/") => Ok(()),
            (Method::Post, "/") => {
                if let Ok(Ok(total_uses)) = req.text().await.map(|n| n.parse::<u32>()) {
                    self.token.remaining_uses = total_uses;
                    self.token.total_uses = total_uses;

                    Ok(())
                } else {
                    Err("Problem creating referral")
                }
            }
            (Method::Post, "/use") => {
                if self.token.remaining_uses > 0 {
                    if let Ok(user) = req.text().await {
                        self.token.remaining_uses -= 1;
                        self.token.used_by.push(user);

                        Ok(())
                    } else {
                        Err("Problem decoding user")
                    }
                } else {
                    Err("Referral code has no more uses remaining")
                }
            }
            _ => Err("Unknown action"),
        };

        self.state
            .storage()
            .put_multiple(&self.token)
            .await
            .unwrap();

        match result {
            Ok(()) => Response::from_json(&json!({
                "ok": true,
                "token": serde_json::to_value(&self.token).ok()
            })),
            Err(message) => {
                Response::error(&json!({ "ok": false, "message": message }).to_string(), 400)
            }
        }
    }
}
