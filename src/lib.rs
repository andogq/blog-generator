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
    query_path = "graphql/query.graphql",
    response_derives = "Debug",
    variables_derives = "Debug"
)]
struct Query;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    // Prepare GraphQL response
    let path = req.path();
    let path = path.split('/').skip(1).collect::<Vec<&str>>();

    let mut variables = query::Variables {
        user: "andogq".to_string(),
        post: 1,
        posts: 0,
    };
    let mut template = "404";

    match *path.get(0).unwrap() {
        "" => {
            // Home page
            variables.posts = 10;
            template = "home";
        }
        "post" => {
            // Post
            if let Some(Ok(post_id)) = path.get(1).map(|id| id.parse::<i64>()) {
                variables.post = post_id;
                template = "post";
            }
        }
        _ => {
            // Other
        }
    };

    // Make request
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
            serde_json::to_string(&Query::build_query(variables))
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
                Some(query::ResponseData {
                    user: Some(user),
                    repository:
                        Some(query::QueryRepository {
                            readme,
                            templates:
                                Some(query::QueryRepositoryTemplates::Tree(
                                    query::QueryRepositoryTemplatesOnTree {
                                        entries: Some(templates),
                                    },
                                )),
                            posts,
                            post,
                        }),
                }),
            ..
        }) = response
            .json::<graphql_client::Response<query::ResponseData>>()
            .await
        {
            let post = post.map(|post| (post.title, render_markdown(&post.body)));

            // Process data here
            let content = json!({
                "name": user.name,
                "profile_picture": user.profile_picture,
                "bio": user.bio,
                "hireable": if user.hireable { "yes" } else { "no" },
                "body": readme.map(|readme| match readme {
                    query::QueryRepositoryReadme::Blob(
                        query::QueryRepositoryReadmeOnBlob { text: Some(text) },
                    ) => render_markdown(&text),
                    _ => "".to_string(),
                }),
                "posts": posts.nodes
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|post| {
                        post.map(|post| json!({
                            "link": format!("/post/{}", post.number),
                            "title": post.title
                        }))
                    })
                    .collect::<Vec<serde_json::Value>>(),
                "pinned": user.pinned_items
                    .nodes
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(
                        |pinned| if let Some(query::QueryUserPinnedItemsNodes::Repository(query::QueryUserPinnedItemsNodesOnRepository {
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
                    .collect::<Vec<serde_json::Value>>(),
                "post": post.clone().map(|post| json!({ "title": post.0, "body": post.1 }))
            });

            let title = match template {
                "home" => user.name.unwrap_or_else(|| "Portfolio".to_string()),
                "post" => post
                    .map(|post| post.0)
                    .unwrap_or_else(|| "Post".to_string()),
                _ => "Portfolio".to_string(),
            };

            let templates = templates
                .into_iter()
                .filter_map(|template| {
                    if let (name, Some(Some(text))) = (
                        template.name,
                        template.object.map(|object| match object {
                            query::QueryRepositoryTemplatesOnTreeEntriesObject::Blob(
                                query::QueryRepositoryTemplatesOnTreeEntriesObjectOnBlob { text },
                            ) => text,
                            _ => None,
                        }),
                    ) {
                        Some((name.replace(".html", ""), text))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<String, String>>();

            if let (Some(core_template), Some(page_template)) =
                (templates.get("core"), templates.get(template))
            {
                if let Some(html) = generate_page(core_template, page_template, &title, content) {
                    return Response::from_html(html);
                }
            }
        }
    }

    Response::error("Not Found", 404)
}
