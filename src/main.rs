use futures::future::join_all;
use tokio::task;

async fn get_html(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url)
        .await?
        .text()
        .await?;
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let visited_url
    let urls = ["https://raymondjcox.com", "https://jiayiwu.me"];
    let mut tasks = vec![];
    for url in &urls {
        tasks.push(task::spawn(get_html(url)));
    }
    // let result = join_all(tasks).await;
    for res in join_all(tasks).await {
        println!("{:?}", res);
    }
    Ok(())
}
