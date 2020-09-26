use ansi_term::Colour;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Repository<'a> {
    name: &'a str,
    pulls_url: &'a str,
}

#[derive(Deserialize)]
struct Pull<'a> {
    title: &'a str,
    number: u32,
}

static CLONE_TOKEN: &'static str = env!("CLONE_TOKEN");

async fn merge_pr(pull_url: &str, number: u32, username: &str) -> Result<(), reqwest::Error> {
    let merge_url = format!("{}/{}/merge", pull_url, number);

    Client::new()
        .put(merge_url.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", username)
        .basic_auth(username, Some(CLONE_TOKEN.clone()))
        .send()
        .await?;

    Ok(())
}

async fn detect_dependabot_prs(
    repo: &Repository<'_>,
    username: &str,
) -> Result<(), reqwest::Error> {
    println!(
        "{}",
        Colour::Yellow
            .italic()
            .paint(format!("Detecting dependabot PRs for {}", repo.name))
    );

    let pull_url = repo.pulls_url.replace("{/number}", "");
    let mut page_no: u32 = 1;
    let depdenda_re: Regex = Regex::new(r"^Bump.*from.*to.*$").unwrap();
    loop {
        let response_text = Client::new()
            .get(pull_url.as_str())
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", username)
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("state", "open"),
                ("per_page", "100"),
                ("page", page_no.to_string().as_str()),
            ])
            .basic_auth(username, Some(CLONE_TOKEN.clone()))
            .send()
            .await?
            .text()
            .await?;

        let pulls: Vec<Pull> = serde_json::from_str(&response_text).unwrap();

        if pulls.len() == 0 {
            break;
        }

        for pull in pulls.iter().filter(|x| depdenda_re.is_match(x.title)) {
            println!(
                "{}",
                Colour::Red.paint(format!("Performing merge of {}", pull.title))
            );
            merge_pr(&pull_url, pull.number, username).await?;
        }

        page_no += 1;
    }

    Ok(())
}

async fn iterate_repos(username: &str) -> Result<(), reqwest::Error> {
    println!(
        "\n{}\n",
        Colour::Blue.paint("Fetching repositories for the user")
    );

    let mut page_no: u32 = 1;
    // TODO: Convert this to iterator and chain it to the
    // pull request iterator in the second place.
    loop {
        let response_text = Client::new()
            .get("https://api.github.com/users/yashladha/repos")
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", username)
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("per_page", "100"),
                ("page", page_no.to_string().as_str()),
            ])
            .basic_auth(username, Some(CLONE_TOKEN.clone()))
            .send()
            .await?
            .text()
            .await?;

        let repo_list: Vec<Repository> = serde_json::from_str(&response_text).unwrap();

        // Break as there are no more PRs to resolve
        if repo_list.len() == 0 {
            break;
        }

        page_no += 1;

        for repo in repo_list.iter() {
            detect_dependabot_prs(repo, username).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let username: String = std::env::args()
        .nth(1)
        .expect("Github username needs to be passed");

    println!(
        "{}",
        Colour::Cyan
            .bold()
            .paint("Clean your dependabot PRs in fastest way")
    );

    iterate_repos(&username).await.unwrap()
}
