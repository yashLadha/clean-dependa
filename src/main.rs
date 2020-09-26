use ansi_term::Colour;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Repository<'a> {
    pulls_url: &'a str,
}

#[derive(Deserialize)]
struct Pull<'a> {
    title: &'a str,
    number: u32,
}

async fn merge_pr(pull_url: &str, number: u32) -> Result<(), reqwest::Error> {
    let merge_url = format!("{}/{}/merge", pull_url, number);
    let clone_token = env!("CLONE_TOKEN");
    Client::new()
        .put(merge_url.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "yashladha")
        .basic_auth("yashladha", Some(clone_token.clone()))
        .send()
        .await?;

    Ok(())
}

async fn detect_dependabot_prs(repo: &Repository<'_>) -> Result<(), reqwest::Error> {
    println!(
        "{}",
        Colour::Yellow.italic().paint("Detecting dependabot PRs")
    );

    let clone_token = env!("CLONE_TOKEN");
    let pull_url = repo.pulls_url.replace("{/number}", "");
    let mut i: u32 = 0;
    let depdenda_re: Regex = Regex::new(r"^Bump.*from.*to.*$").unwrap();
    loop {
        let response = Client::new()
            .get(pull_url.as_str())
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "yashladha")
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("state", "open"),
                ("per_page", "100"),
                ("page", i.to_string().as_str()),
            ])
            .basic_auth("yashladha", Some(clone_token.clone()))
            .send()
            .await?;

        let response_text = response.text().await?;
        let pulls: Vec<Pull> = serde_json::from_str(&response_text).unwrap();

        if pulls.len() == 0 {
            break;
        }

        for pull in pulls.iter().filter(|x| depdenda_re.is_match(x.title)) {
            println!(
                "{}",
                Colour::Red.paint(format!("Performing merge of {}", pull.title))
            );
            merge_pr(&pull_url, pull.number).await?;
        }

        i += 1;
    }

    Ok(())
}

async fn iterate_repos() -> Result<(), reqwest::Error> {
    println!(
        "\n\n{}\n",
        Colour::Blue.paint("Fetching repositories for the user")
    );

    let clone_token = env!("CLONE_TOKEN");
    let mut i = 0;
    // TODO: Convert this to iterator and chain it to the
    // pull request iterator in the second place.
    loop {
        let response = Client::new()
            .get("https://api.github.com/users/yashladha/repos")
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "yashladha")
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("per_page", "100"),
                ("page", i.to_string().as_str()),
            ])
            .basic_auth("yashladha", Some(clone_token.clone()))
            .send()
            .await?;

        let response_text = response.text().await?;
        let repo_list: Vec<Repository> = serde_json::from_str(&response_text).unwrap();

        // Break as there are no more PRs to resolve
        if repo_list.len() == 0 {
            break;
        }

        i += 1;

        for repo in repo_list.iter() {
            detect_dependabot_prs(repo).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!(
        "{}",
        Colour::Cyan
            .bold()
            .paint("Clean your dependabot PRs in fastest way")
    );

    iterate_repos().await;
}
