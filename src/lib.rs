use github::Repo;
use pulldown_cmark::{Options, Parser};
use worker::*;

use crate::utils::Source;

mod github;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

static TEMPLATE: &str = include_str!("../template.html");

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_, _| async {
            if let Some(user) = github::User::get("andogq").await {
                let mut res = TEMPLATE.to_string();

                for (key, value) in user.get_key_values() {
                    res = res.replace(&format!("{{{{{}}}}}", key), &value);
                }

                let repo = Repo {
                    name: "andogq/blog".to_string(),
                };

                let issues = repo.get_issues().await;

                let mut options = Options::empty();
                options.insert(Options::ENABLE_TABLES);
                options.insert(Options::ENABLE_TASKLISTS);
                options.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = Parser::new_ext(&issues[0].body, options);

                let mut rendered = String::new();
                pulldown_cmark::html::push_html(&mut rendered, parser);
                res = res.replace("{{posts}}", &rendered);

                Response::from_html(res)
            } else {
                Response::error("Not found", 404)
            }
        })
        .run(req, env)
        .await
}
