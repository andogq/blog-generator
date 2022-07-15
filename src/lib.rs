use github::{File, Repo};
use handlebars::Handlebars;
use pulldown_cmark::{Options, Parser};
use serde_json::json;
use worker::*;

mod github;
mod utils;

async fn generate_page(
    repo: &Repo,
    page: &str,
    title: &str,
    content: serde_json::Value,
) -> Option<String> {
    // Download template and core template from github
    // TODO Could make these in parallel
    if let (Some(core_template), Some(page_template)) = (
        File::new(&repo.get_contents_path("/templates/core.html"))
            .get_contents()
            .await,
        File::new(&repo.get_contents_path(&format!("/templates/{}.html", page)))
            .get_contents()
            .await,
    ) {
        let handlebars = Handlebars::new();

        if let Ok(html) = handlebars.render_template(&page_template, &content) {
            if let Ok(html) = handlebars.render_template(
                &core_template,
                &json!({
                    "title": title,
                    "content": html
                }),
            ) {
                return Some(html);
            }
        }
    }

    None
}

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
        .get_async("/", |_, RouteContext { env, .. }| async move {
            if let Some(user) = github::User::get("andogq").await {
                if let Some(repo) = user.get_repo("blog".to_string()).await {
                    let issues = repo.get_issues().await;

                    let pinned_repositories = user
                        .get_pinned(&env.secret("gh_key").unwrap().to_string())
                        .await;

                    let content = json!({
                        "name": user.name,
                        "profile_picture": user.profile_picture,
                        "bio": user.bio,
                        "hireable": if user.hireable { "yes" } else { "no" },
                        "posts": issues.iter().map(|post| json!({
                            "link": format!("/posts/{}", post.id),
                            "title": post.title
                        })).collect::<Vec<serde_json::Value>>(),
                        "pinned": pinned_repositories.iter().map(|repo| json!({
                            "name": repo.name,
                            "description": repo.description,
                            "languages": repo.languages.join(", ")
                        })).collect::<Vec<serde_json::Value>>()
                    });

                    if let Some(html) = generate_page(&repo, "home", &user.name, content).await {
                        return Response::from_html(html);
                    }
                }
            }

            Response::error("Internal error", 500)
        })
        .get_async("/posts/:id", |_, ctx| async move {
            if let Ok(id) = ctx.param("id").unwrap().parse::<u32>() {
                if let Some(user) = github::User::get("andogq").await {
                    if let Some(repo) = user.get_repo("blog".to_string()).await {
                        if let Some(issue) = repo.get_issue(id).await {
                            let mut options = Options::empty();
                            options.insert(Options::ENABLE_TABLES);
                            options.insert(Options::ENABLE_TASKLISTS);
                            options.insert(Options::ENABLE_STRIKETHROUGH);
                            let parser = Parser::new_ext(&issue.body, options);

                            let mut rendered = String::new();
                            pulldown_cmark::html::push_html(&mut rendered, parser);

                            let content = json!({
                                "post_title": issue.title,
                                "post_body": rendered
                            });

                            if let Some(page) =
                                generate_page(&repo, "post", &issue.title, content).await
                            {
                                return Response::from_html(page);
                            }
                        }
                    }
                }
            }

            Response::error("Internal error", 500)
        })
        .run(req, env)
        .await
}
