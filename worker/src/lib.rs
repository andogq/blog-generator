use std::collections::HashMap;

use graphql_client::GraphQLQuery;
use handlebars::Handlebars;
use pulldown_cmark::{Options, Parser};
use serde_json::json;
use worker::*;

pub mod durable_objects;
mod utils;

fn generate_page(
    core_template: &str,
    page_template: &str,
    title: &str,
    content: serde_json::Value,
) -> Option<String> {
    let handlebars = Handlebars::new();

    match handlebars.render_template(page_template, &content) {
        Ok(html) => {
            if let Ok(html) = handlebars.render_template(
                core_template,
                &json!({
                    "title": title,
                    "content": html,
                    "page_variables": content
                }),
            ) {
                return Some(html);
            }
        }
        Err(e) => console_log!("{:?}", e),
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
type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github.schema.graphql",
    query_path = "graphql/query.graphql"
)]
struct Query;

const DURABLE_OBJECT_WHITELIST: &[&str] = &["referral_code"];

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    let path = req
        .url()
        .map(|url| url.path().to_string())
        .unwrap_or_default();

    let mut path = path.split('/').map(|s| s.to_string()).skip(1);

    if path.next().map(|base| base == "_").unwrap_or(false) {
        // In API section
        // Cheack the Authorization header
        if let Ok(Some(authorization_header)) = req.headers().get("Authorization") {
            if let (Some("Bearer"), Some(token)) = {
                let mut iter = authorization_header.split(' ');
                (iter.next(), iter.next())
            } {
                let expected_server = env
                    .secret("SERVER_TOKEN")
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if token == expected_server {
                    return match path.next().unwrap_or_default().as_str() {
                        "kv" => {
                            if let (Some(store_name), Some(key)) = (path.next(), path.next()) {
                                if let Ok(store) = env.kv(&store_name) {
                                    console_log!("{}", key);
                                    if let Some::<String>(res) = match req.method() {
                                        Method::Get => store.get(&key).text().await.unwrap_or(None),
                                        Method::Post => {
                                            if let Ok(body) = req.text().await {
                                                if let Ok(put) = store.put(&key, body) {
                                                    if put.execute().await.is_ok() {
                                                        Some("ok".to_string())
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    } {
                                        return Response::ok(res);
                                    }
                                }
                            }

                            Response::error("Bad request", 400)
                        }
                        "do" => {
                            if let (Some(object), Some(instance_name), path) = (
                                path.next(),
                                path.next(),
                                path.collect::<Vec<String>>().join("/"),
                            ) {
                                if DURABLE_OBJECT_WHITELIST.contains(&object.as_str()) {
                                    console_log!("In whitelist, {}", object.to_uppercase());

                                    if let Ok(Ok(Ok(stub))) =
                                        env.durable_object(&object.to_uppercase()).map(|object| {
                                            object
                                                .id_from_name(&instance_name)
                                                .map(|instance| instance.get_stub())
                                        })
                                    {
                                        console_log!("found stab");
                                        return stub
                                            .fetch_with_request(
                                                Request::new_with_init(
                                                    &format!("http://localhost/{}", path),
                                                    &{
                                                        let mut request_init = RequestInit::new();

                                                        let method = req.method();
                                                        request_init.with_method(method.clone());
                                                        if method != Method::Get
                                                            && method != Method::Head
                                                        {
                                                            request_init.with_body(
                                                                req.text()
                                                                    .await
                                                                    .ok()
                                                                    .map(|body| body.into()),
                                                            );
                                                        }
                                                        request_init
                                                    },
                                                )
                                                .unwrap(),
                                            )
                                            .await;
                                    }
                                }
                            }

                            Response::error("Bad request", 400)
                        }
                        _ => Response::error("Not found", 404),
                    };
                }
            }
        }

        Response::error("Not authorized", 403)
    } else {
        // Load KV store
        if let (Ok(domains_store), Ok(Some(domain))) = (
            env.kv("domains"),
            req.url()
                .map(|url| url.domain().map(|domain| domain.to_string())),
        ) {
            if let Ok(Some(user)) = domains_store.get(&domain).text().await {
                console_log!("{}", user);

                // Prepare GraphQL response
                let path = req.path();
                let path = path.split('/').skip(1).collect::<Vec<&str>>();

                let mut variables = query::Variables {
                    user,
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
                                &format!("Bearer {}", env.secret("GH_KEY").unwrap().to_string()),
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
                    Request::new_with_init("https://api.github.com/graphql", &request_init)
                        .unwrap(),
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
                        let user_name = user.name.unwrap_or_default();
                        let post_title = post.as_ref().map(|post| post.title.to_owned());

                        // Process data here
                        let content = json!({
                            "user": json!({
                                "name": user_name,
                                "profile_picture": user.profile_picture,
                                "email": user.email,
                                "bio": user.bio,

                                "location": user.location,
                                "hireable": user.hireable,
                                "company": user.company,

                                "github_profile": user.url,
                                "twitter_profile": user.twitter_username.map(|username| format!("https://twitter.com/{}", username)),

                                "followers": user.followers.total_count,
                                "following": user.following.total_count,
                            }),
                            "readme": readme.map(|readme| match readme {
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
                                        "title": post.title,
                                        "link": format!("/post/{}", post.number),
                                        "labels": post.labels
                                            .map(|labels| labels.nodes
                                                 .unwrap_or_default()
                                                 .into_iter()
                                                 .filter_map(|label| label.map(|label| json!({
                                                     "name": label.name,
                                                     "color": label.color
                                                 })))
                                                .collect::<Vec<serde_json::Value>>(),
                                            )
                                            .unwrap_or_default(),
                                        "created": post.created_at,
                                        "updated": post.updated_at
                                    }))
                                })
                                .collect::<Vec<serde_json::Value>>(),
                            "pinned": user.pinned_items
                                .nodes
                                .unwrap_or_default()
                                .into_iter()
                                .filter_map(
                                    |pinned| if let Some(query::QueryUserPinnedItemsNodes::Repository(query::QueryUserPinnedItemsNodesOnRepository {name,description,homepage_url,github_url,languages:Some(languages), fork_count, stargazer_count })) = pinned {
                                        Some(json!({
                                            "name": name,
                                            "description": description,
                                            "languages": languages.nodes
                                                .unwrap_or_default()
                                                .into_iter()
                                                .filter_map(|language| language.map(|language| language.name))
                                                .collect::<Vec<String>>(),
                                            "homepage": homepage_url,
                                            "stargazers": stargazer_count,
                                            "forks": fork_count,
                                            "github_url": github_url
                                        }))
                                    } else { None }
                                )
                                .collect::<Vec<serde_json::Value>>(),
                            "post": post.map(|post| json!({
                                "title": post.title,
                                "body": render_markdown(&post.body),
                                "labels": post.labels
                                    .map(|labels| labels.nodes
                                         .unwrap_or_default()
                                         .into_iter()
                                         .filter_map(|label| label.map(|label| json!({
                                             "name": label.name,
                                             "color": label.color
                                         })))
                                        .collect::<Vec<serde_json::Value>>(),
                                    )
                                    .unwrap_or_default(),
                                "created": post.created_at,
                                "updated": post.updated_at
                            })),
                            "page": template
                        });

                        let title = match template {
                            "home" => user_name,
                            "post" => post_title.unwrap_or_else(|| "Post".to_string()),
                            _ => "Portfolio".to_string(),
                        };

                        let templates = templates
                            .into_iter()
                            .filter_map(|template| {
                                if let (name, Some(Some(text))) = (
                                    template.name,
                                    template.object.map(|object| {
                                        match object {
                                    query::QueryRepositoryTemplatesOnTreeEntriesObject::Blob(
                                        query::QueryRepositoryTemplatesOnTreeEntriesObjectOnBlob {
                                            text,
                                        },
                                    ) => text,
                                    _ => None,
                                }
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
                            if let Some(html) =
                                generate_page(core_template, page_template, &title, content)
                            {
                                Response::from_html(html)
                            } else {
                                Response::error("Problem generating template", 500)
                            }
                        } else {
                            Response::error("Problem getting template", 500)
                        }
                    } else {
                        Response::error("Problem with decoding graphql", 500)
                    }
                } else {
                    Response::error("Problem with graphql", 500)
                }
            } else {
                Response::error("Domain not found", 400)
            }
        } else {
            Response::error("Internal Error", 500)
        }
    }
}
