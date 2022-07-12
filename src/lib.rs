use github::{File, Repo};
use pulldown_cmark::{Options, Parser};
use utils::replace;
use worker::*;

mod github;
mod utils;

async fn generate_page(
    repo: &Repo,
    page: &str,
    title: &str,
    content: Vec<(&str, &str)>,
) -> Option<String> {
    // Download template and core template from github
    // TODO Could make these in parallel
    if let (Some(mut core_template), Some(mut page_template)) = (
        File::new(&repo.get_contents_path("/templates/core.html"))
            .get_contents()
            .await,
        File::new(&repo.get_contents_path(&format!("/templates/{}.html", page)))
            .get_contents()
            .await,
    ) {
        // Fill content
        for (key, value) in content.iter() {
            page_template = replace(page_template, key, value);
        }

        core_template = replace(core_template, "content", &page_template);
        core_template = replace(core_template, "title", title);

        Some(core_template)
    } else {
        None
    }
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
        .get_async("/", |_, _| async {
            if let Some(user) = github::User::get("andogq").await {
                let repo = Repo {
                    user: "andogq".to_string(),
                    name: "blog".to_string(),
                };

                let issues = repo.get_issues().await;
                let posts = issues
                    .iter()
                    .map(|post| {
                        format!("<li><a href=\"/posts/{}\">{}</a></li>", post.id, post.title)
                    })
                    .collect::<String>();

                let content: Vec<(&str, &str)> = vec![
                    ("name", &user.name),
                    ("profile_picture", &user.profile_picture),
                    ("bio", &user.bio),
                    ("hireable", if user.hireable { "yes" } else { "no" }),
                    ("posts", &posts),
                ];

                if let Some(page) = generate_page(&repo, "home", &user.name, content).await {
                    Response::from_html(page)
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

                    let content: Vec<(&str, &str)> =
                        vec![("post_title", &issue.title), ("post_body", &rendered)];

                    if let Some(page) = generate_page(&repo, "post", &issue.title, content).await {
                        Response::from_html(page)
                    } else {
                        Response::error("Internal error", 500)
                    }
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
