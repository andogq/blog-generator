use github::{File, Repo};
use pulldown_cmark::{Options, Parser};
use utils::replace;
use worker::*;

mod github;
mod utils;

enum ContentType {
    String(String),
    List(Vec<Vec<(String, String)>>),
}

async fn generate_page(
    repo: &Repo,
    page: &str,
    title: &str,
    content: Vec<(String, ContentType)>,
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
            page_template = match value {
                ContentType::String(value) => replace(page_template, key, value),
                ContentType::List(iterations) => {
                    if let (Some(template_start), Some(template_end)) = (
                        page_template.find(&format!("{{{{{}}}}}", key)),
                        page_template
                            .find(&format!("{{{{/{}}}}}", key))
                            .map(|i| i + key.len() + 5),
                    ) {
                        let section_template = page_template
                            [(template_start + key.len() + 4)..(template_end - (key.len() + 5))]
                            .to_owned();
                        page_template.replace_range(
                            template_start..template_end,
                            &iterations
                                .iter()
                                .map(|replacements| {
                                    replacements.iter().fold(
                                        section_template.to_owned(),
                                        |template, (key, value)| replace(template, key, value),
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        );
                        page_template
                    } else {
                        page_template
                    }
                }
            }
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
        .get_async("/", |_, RouteContext { env, .. }| async move {
            if let Some(user) = github::User::get("andogq").await {
                if let Some(repo) = user.get_repo("blog".to_string()).await {
                    let issues = repo.get_issues().await;
                    let posts = issues
                        .iter()
                        .map(|post| {
                            format!("<li><a href=\"/posts/{}\">{}</a></li>", post.id, post.title)
                        })
                        .collect::<String>();

                    let pinned_repositories = user
                        .get_pinned(&env.secret("gh_key").unwrap().to_string())
                        .await;

                    let mut content: Vec<(String, ContentType)> = vec![
                        ("name", &user.name as &str),
                        ("profile_picture", &user.profile_picture),
                        ("bio", &user.bio),
                        ("hireable", if user.hireable { "yes" } else { "no" }),
                        ("posts", &posts),
                    ]
                    .iter()
                    .map(|(key, value)| (key.to_string(), ContentType::String(value.to_string())))
                    .collect();

                    content.push((
                        "pinned".to_string(),
                        ContentType::List(
                            pinned_repositories
                                .into_iter()
                                .map(|repo| {
                                    vec![
                                        ("repo.name".to_string(), repo.name),
                                        (
                                            "repo.description".to_string(),
                                            repo.description.unwrap_or_else(|| "".to_string()),
                                        ),
                                        ("repo.languages".to_string(), repo.languages.join(", ")),
                                    ]
                                })
                                .collect(),
                        ),
                    ));

                    if let Some(page) = generate_page(&repo, "home", &user.name, content).await {
                        Response::from_html(page)
                    } else {
                        Response::error("Internal error", 500)
                    }
                } else {
                    Response::error("Not found", 404)
                }
            } else {
                Response::error("Internal error", 500)
            }
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

                            let content: Vec<(&str, &str)> =
                                vec![("post_title", &issue.title), ("post_body", &rendered)];

                            if let Some(page) = generate_page(
                                &repo,
                                "post",
                                &issue.title,
                                content
                                    .iter()
                                    .map(|(key, value)| {
                                        (key.to_string(), ContentType::String(value.to_string()))
                                    })
                                    .collect(),
                            )
                            .await
                            {
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
                } else {
                    Response::error("Internal error", 500)
                }
            } else {
                Response::error("Internal error", 500)
            }
        })
        .run(req, env)
        .await
}
