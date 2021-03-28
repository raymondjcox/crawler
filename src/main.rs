use url::Url;
use std::collections::HashSet;
use futures::future::join_all;
use regex::Regex;
use tokio::task;

async fn get_html(url: String) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let html = reqwest::get(url.clone())
        .await?
        .text()
        .await?;
    Ok((url, html))
}


fn get_links(html: &str) -> Vec<String> {
    let anchor_tags = Regex::new(r#"a href="(.*?)""#).unwrap();
    return anchor_tags.captures_iter(&html).map(|cap| String::from(cap.get(1).unwrap().as_str())).collect()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let banned = ["linkedin", "twitter", "instagram", "facebook", "drift", "github", "google", "aws", "wikipedia"];
    let mut visited_urls = HashSet::new();
    let mut urls: Vec<String> = vec![String::from("https://jiayiwu.me")];

    while !urls.is_empty() {
        let mut tasks = Vec::new();
        println!("Number of sites analyzed: {}", visited_urls.len());
        while !urls.is_empty() {
            let url = urls.pop().unwrap();
            visited_urls.insert(url.clone());
            tasks.push(task::spawn(get_html(url)));
        }
        for html in join_all(tasks).await {
            let h = html.unwrap();
            match h {
                Ok((url, html)) => {
                    println!("Finding links for {}", url);
                    let links = get_links(&html);
                    for link in links {
                        if visited_urls.contains(&link) || banned.iter().any(|b| link.contains(b)) {
                            continue;
                        }
                        if link.starts_with("http") {
                            urls.push(link.clone());
                        } else {
                            let parsed_url = String::from(Url::parse(&url).unwrap().join(&link).unwrap().as_str());
                            if visited_urls.contains(&parsed_url) {
                                continue;
                            }
                            visited_urls.insert(parsed_url.clone());
                            urls.push(parsed_url);
                        }
                    }
                },
                Err(_a) => continue
            }
        }
    }
    Ok(())
}
