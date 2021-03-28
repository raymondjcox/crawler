use url::Url;
use std::collections::HashSet;
use futures::future::join_all;
use regex::Regex;
use tokio::task;

const DOMAIN: &str = "google.com";
const START_URL: &str = "https://www.google.com";

fn get_links_from_html(url: &str, html: &str) -> Vec<String> {
    let parsed_url = Url::parse(&url).unwrap();
    let anchor_tags = Regex::new(r#"a href="(.*?)""#).unwrap();
    return anchor_tags.captures_iter(&html).map(|cap| String::from(parsed_url.join(cap.get(1).unwrap().as_str()).unwrap().as_str())).collect()
}


async fn get_links(url: String) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let html = reqwest::get(url.clone())
        .await?
        .text()
        .await?;
    let links = task::spawn_blocking(move ||get_links_from_html(&url, &html)).await?;
    Ok(links)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut visited_urls = HashSet::new();
    let mut num_visited = 0;
    let mut urls: Vec<String> = vec![String::from(START_URL)];

    while !urls.is_empty() {
        let mut tasks = Vec::new();
        let mut urls_to_visit = Vec::new();
        println!("Number of sites visited: {}", num_visited);
        while !urls.is_empty() && tasks.len() < 8 {
            let url = urls.remove(0);
            urls_to_visit.push(url.clone());
            tasks.push(task::spawn(get_links(url)));
            num_visited += 1;
        }
        println!("Visiting {} urls: {:?}", urls_to_visit.len(), urls_to_visit);

        for links in join_all(tasks).await {
            match links.unwrap() {
                Ok(links) => {
                    for link in links {
                        if visited_urls.contains(&link) || !link.contains(DOMAIN) {
                            continue;
                        }
                        visited_urls.insert(link.clone());
                        urls.push(link);
                    }
                },
                Err(e) => println!("{:?}", e)
            }
        }
    }
    Ok(())
}
