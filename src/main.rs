use url::Url;
use structopt::StructOpt;
use std::collections::HashSet;
use futures::future::join_all;
use regex::Regex;
use tokio::task;


fn get_links_from_html(url: &str, html: &str) -> Vec<String> {
    let parsed_url = Url::parse(&url).unwrap();
    let anchor_tags = Regex::new(r#"(?s)<a.*?href="(.*?)"(.*?)</a>"#).unwrap();
    return anchor_tags.captures_iter(&html).map(|cap| parsed_url.join(cap.get(1).unwrap().as_str()).unwrap().to_string()).collect()
}


async fn get_links(url: String) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let html = reqwest::get(&url)
        .await?
        .text()
        .await?;
    let links = task::spawn_blocking(move ||get_links_from_html(&url, &html)).await?;
    Ok(links)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "crawler", about = "Crawls all links on a webpage and prints the results.")]
struct Opt {
    #[structopt(short, long)]
    verbose: bool,
    start_url: String,
}

fn get_domain_with_scheme(url: &str) -> String {
    let parsed_url = Url::parse(url).unwrap();
    format!("{}://{}", parsed_url.scheme(), parsed_url.domain().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut visited_urls = HashSet::new();
    let mut num_visited = 0;
    let domain_with_scheme = get_domain_with_scheme(&opt.start_url);
    let mut urls: Vec<String> = vec![opt.start_url];

    while !urls.is_empty() {
        let mut tasks = Vec::new();
        println!("Number of links visited: {}", num_visited);
        while !urls.is_empty() && tasks.len() < 8 {
            tasks.push(task::spawn(get_links(urls.remove(0))));
            num_visited += 1;
        }

        for links in join_all(tasks).await {
            match links.unwrap() {
                Ok(links) => {
                    for link in links {
                        if visited_urls.contains(&link) || !link.starts_with(&domain_with_scheme) {
                            continue;
                        }
                        if opt.verbose {
                            println!("{}", link);
                        }
                        visited_urls.insert(link.to_string());
                        urls.push(link);
                    }
                },
                Err(e) => println!("{:?}", e)
            }
        }
    }
    println!("Total pages: {}", visited_urls.len());
    for url in visited_urls {
        println!("{}", url);
    }
    Ok(())
}
