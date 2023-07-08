use std::ops::Deref;

use feed_rs::parser::parse;
use scraper::{Html, Selector};
use url::Url;

use crate::HttpClients;

pub struct Post {
    pub title: Option<String>,
    pub description: String,
    pub url: Url,
}

impl Post {
    pub async fn get_latest(clients: &HttpClients) -> reqwest::Result<Self> {
        let post_url = latest_post_from_rss(clients).await?;

        let desc_selector = Selector::parse("meta[name=\"description\"]").unwrap();
        let title_selector = Selector::parse("title").unwrap();
        let post = Html::parse_document(
            &clients
                .html
                .get(post_url.clone())
                .send()
                .await?
                .text()
                .await?,
        );

        let desc_element = post
            .select(&desc_selector)
            .next()
            .expect("Could not find 'meta' element with name 'description'");
        let title_element = post.select(&title_selector).next();

        Ok(Self {
            title: title_element.map(|title| title.text().collect::<Vec<_>>().join("")),
            description: desc_element
                .value()
                .attr("content")
                .expect("Invalid formatting for 'name' meta tag")
                .to_string(),
            url: post_url,
        })
    }
}

async fn latest_post_from_rss(clients: &HttpClients) -> reqwest::Result<Url> {
    let rss_response = clients
        .html
        .get(&clients.website_rss_url) // https://www.wildfly.org/feed.xml
        .send()
        .await?
        .bytes()
        .await?;
    let parsed_feed =
        parse(rss_response.deref()).expect("Unable to parse team-role-org-testing feed");
    let post = parsed_feed.entries.first().expect("No posts found in feed");

    Ok(Url::parse(
        post.links
            .first()
            .expect("No link provided with first post")
            .href
            .as_str(),
    )
    .unwrap())
}
