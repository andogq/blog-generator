use std::collections::HashMap;

use graphql_client::GraphQLQuery;
use handlebars::Handlebars;
use pulldown_cmark::{Options, Parser};
use serde_json::json;
use worker::*;

mod utils;

fn generate_page(
    core_template: &str,
    page_template: &str,
    title: &str,
    content: serde_json::Value,
) -> Option<String> {
    let handlebars = Handlebars::new();

    if let Ok(html) = handlebars.render_template(page_template, &content) {
        if let Ok(html) = handlebars.render_template(
            core_template,
            &json!({
                "title": title,
                "content": html
            }),
        ) {
            return Some(html);
        }
    }

    None
}

fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut rendered = String::new();
    pulldown_cmark::html::push_html(&mut rendered, parser);

    rendered
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

#[allow(clippy::upper_case_acronyms)]
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github.schema.graphql",
    query_path = "graphql/home.query.graphql",
    response_derives = "Debug"
)]
struct HomeQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github.schema.graphql",
    query_path = "graphql/post.query.graphql",
    response_derives = "Debug"
)]
struct PostQuery;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_, RouteContext { env, .. }| async move {
            let variables = home_query::Variables {
                user: "andogq".to_string(),
            };

            let request_init = {
                let mut init = RequestInit::new();
                init.with_headers({
                    let mut headers = Headers::new();
                    headers.set("User-Agent", "rust").unwrap();
                    headers
                        .set(
                            "Authorization",
                            &format!("Bearer {}", env.secret("gh_key").unwrap().to_string()),
                        )
                        .unwrap();
                    headers
                })
                .with_method(Method::Post)
                .with_body(
                    serde_json::to_string(&HomeQuery::build_query(variables))
                        .ok()
                        .map(|val| val.into()),
                );

                init
            };

            if let Ok(mut response) = Fetch::Request(
                Request::new_with_init("https://api.github.com/graphql", &request_init).unwrap(),
            )
            .send()
            .await
            {
                if let Ok(graphql_client::Response {
                    data:
                        Some(home_query::ResponseData {
                            user: Some(user),
                            repository: Some(repository),
                        }),
                    ..
                }) = response
                    .json::<graphql_client::Response<home_query::ResponseData>>()
                    .await
                {
                    let templates = repository.templates.map(|templates| match templates {
                        home_query::HomeQueryRepositoryTemplates::Tree(
                            home_query::HomeQueryRepositoryTemplatesOnTree {
                                entries: Some(templates),
                            },
                        ) => templates.into_iter().filter_map(|template| {
                            if let home_query::HomeQueryRepositoryTemplatesOnTreeEntries {
                                name,
                                object: Some(home_query::HomeQueryRepositoryTemplatesOnTreeEntriesObject::Blob(home_query::HomeQueryRepositoryTemplatesOnTreeEntriesObjectOnBlob {
                                    text: Some(template)
                                }))
                            } = template {
                                Some((name.replace(".html", ""), template))
                            } else { None }
                        }).collect::<HashMap<String, String>>(),
                        _ => HashMap::new()
                    }).unwrap_or_default();

                    let content = json!({
                        "name": user.name,
                        "profile_picture": user.profile_picture,
                        "bio": user.bio,
                        "hireable": if user.hireable { "yes" } else { "no" },
                        "body": repository.readme.map(|readme| match readme {
                            home_query::HomeQueryRepositoryReadme::Blob(
                                home_query::HomeQueryRepositoryReadmeOnBlob { text: Some(text) },
                            ) => render_markdown(&text),
                            _ => "".to_string(),
                        }),
                        "posts": repository.posts.nodes
                            .unwrap_or_default()
                            .into_iter()
                            .filter_map(|post| {
                                post.map(|post| json!({
                                    "link": format!("/posts/{}", post.number),
                                    "title": post.title
                                }))
                            })
                            .collect::<Vec<serde_json::Value>>(),
                        "pinned": user.pinned_items
                            .nodes
                            .unwrap_or_default()
                            .into_iter()
                            .filter_map(
                                |pinned| if let Some(home_query::HomeQueryUserPinnedItemsNodes::Repository(home_query::HomeQueryUserPinnedItemsNodesOnRepository {
                                    name,
                                    description,
                                    languages: Some(languages)
                                })) = pinned {
                                    Some(json!({
                                        "name": name,
                                        "description": description,
                                        "languages": languages.nodes
                                            .unwrap_or_default()
                                            .into_iter()
                                            .filter_map(|language| language.map(|language| language.name))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    }))
                                } else { None }
                            )
                            .collect::<Vec<serde_json::Value>>()
                    });

                    if let (Some(core_template), Some(home_template)) = (templates.get("core"), templates.get("home")) {
                        if let Some(html) = generate_page(core_template, home_template, &user.name.unwrap_or_else(|| "Portfolio".to_string()), content) {
                            return Response::from_html(html);
                        }
                    }
                }
            }

            Response::error("Internal error", 500)
        })
        .get_async("/posts/:id", |_, ctx| async move {
            if let Ok(id) = ctx.param("id").unwrap().parse::<u32>() {
                let variables = post_query::Variables {
                    user: "andogq".to_string(),
                    post: id as i64
                };

                let request_init = {
                    let mut init = RequestInit::new();
                    init.with_headers({
                        let mut headers = Headers::new();
                        headers.set("User-Agent", "rust").unwrap();
                        headers
                            .set(
                                "Authorization",
                                &format!("Bearer {}", ctx.env.secret("gh_key").unwrap().to_string()),
                            )
                            .unwrap();
                        headers
                    })
                    .with_method(Method::Post)
                    .with_body(
                        serde_json::to_string(&PostQuery::build_query(variables))
                            .ok()
                            .map(|val| val.into()),
                    );

                    init
                };

                if let Ok(mut response) = Fetch::Request(
                    Request::new_with_init("https://api.github.com/graphql", &request_init).unwrap(),
                )
                .send()
                .await
                {
                    if let Ok(graphql_client::Response {
                        data:
                            Some(post_query::ResponseData {
                                repository: Some(post_query::PostQueryRepository {
                                    issue: Some(post)
                                })
                            }),
                        ..
                    }) = response
                        .json::<graphql_client::Response<post_query::ResponseData>>()
                        .await {
                        let _content = json!({
                            "post_title": post.title,
                            "post_body": render_markdown(&post.body)
                        });

                        return Response::from_html(render_markdown(&post.body));
                    }
                }
            }

            Response::error("Internal error", 500)
        })
        .run(req, env)
        .await
}
