use std::time::Duration;

use feed_rs::model::Link;
use feed_rs::parser::parse;
use octocrab::Octocrab;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Unable to build GitHub API client");

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Unable to build HTTP client");

    let post_url  = latest_post(&client).await;

    // TODO:
    //   - Strip to the part of the URL that becomes the
    //   - get the first paragraph? or not
    //   - add link to body of request

}

async fn latest_post(rss_client: &Client) -> String {
    let feed = rss_client
        .get("https://team-role-org-testing.github.io/feed.xml") // https://www.wildfly.org/feed.xml
        .send().await.unwrap()
        .bytes().await.unwrap();

    let posts = parse(&feed[..]).expect("Unable to parse team-role-org-testing feed");

    posts
        .entries.first().expect("No posts found in feed")
        .links.first().expect("No link provided with first post").href.clone()
}
