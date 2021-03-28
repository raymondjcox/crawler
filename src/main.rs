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
    let banned = ["linkedin", "twitter", "instagram", "facebook", "drift", "github"];
    let mut visited_urls = HashSet::new();
    let mut urls: Vec<String> = vec![String::from("https://jiayiwu.me")];

    while !urls.is_empty() {
        let mut tasks = Vec::new();
        while !urls.is_empty() {
            let url = urls.pop().unwrap();
            println!("visiting {}", url);
            visited_urls.insert(url.clone());
            tasks.push(task::spawn(get_html(url)));
        }
        for html in join_all(tasks).await {
            let h = html.unwrap().unwrap();
            let url = h.0;
            let links = get_links(&h.1);
            // println!("found these links {:?}", links);
            for link in links {
                if link.starts_with("http") && !visited_urls.contains(&link) && !banned.iter().any(|b| link.contains(b)) {
                    urls.push(link.clone());
                }

                if !link.starts_with("http") {
                    urls.push(format!("{}{}", url, link));
                }
            }
        }
    }
    Ok(())
}
