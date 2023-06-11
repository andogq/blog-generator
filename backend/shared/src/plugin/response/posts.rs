use serde::Serialize;

#[derive(Serialize)]
pub struct PostResponse {
    pub number: usize,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub original_link: String,
}

pub type PostsResponse = Vec<PostResponse>;
