use github::Repo;
use pulldown_cmark::{Options, Parser};
use utils::replace;
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

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_, _| async {
            if let Some(user) = github::User::get("andogq").await {
                let repo = Repo {
                    user: "andogq".to_string(),
                    name: "blog".to_string(),
                };

                let templates = repo.get_files("/templates").await;

                if let Some(template) = templates.get("home.html") {
                    if let Some(mut content) = template.get_contents().await {
                        for (key, value) in user.get_key_values() {
                            content = replace(&content, &key, &value);
                        }

                        let issues = repo.get_issues().await;
                        let posts = issues
                            .iter()
                            .map(|post| {
                                format!(
                                    "<li><a href=\"/posts/{}\">{}</a></li>",
                                    post.id, post.title
                                )
                            })
                            .collect::<String>();

                        content = content.replace("{{posts}}", &posts);

                        if let Some(page_template) = templates.get("core.html") {
                            if let Some(mut page) = page_template.get_contents().await {
                                page = page.replace("{{content}}", &content);
                                page = page.replace("{{title}}", &user.name);
                                Response::from_html(page)
                            } else {
                                Response::error("Internal error", 500)
                            }
                        } else {
                            Response::error("Internal error", 500)
                        }
                    } else {
                        Response::error("Internal error", 500)
                    }
                } else {
                    Response::error("Internal error", 500)
                }
            } else {
                Response::error("Not found", 404)
            }
        })
        .get_async("/posts/:id", |_, ctx| async move {
            if let Ok(id) = ctx.param("id").unwrap().parse::<u32>() {
                let repo = Repo {
                    user: "andogq".to_string(),
                    name: "blog".to_string(),
                };

                if let Some(issue) = repo.get_issue(id).await {
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_TABLES);
                    options.insert(Options::ENABLE_TASKLISTS);
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    let parser = Parser::new_ext(&issue.body, options);

                    let mut rendered = String::new();
                    pulldown_cmark::html::push_html(&mut rendered, parser);

                    Response::from_html(rendered)
                } else {
                    Response::error("Not found", 404)
                }
            } else {
                Response::error("Not found", 404)
            }
        })
        .run(req, env)
        .await
}
