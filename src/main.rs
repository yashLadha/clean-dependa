use ansi_term::Colour;
use futures::future::join_all;
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

static GITHUB_TOKEN: &'static str = env!("GITHUB_TOKEN", "Please set the GITHUB_TOKEN value");

struct GithubHandler<'a> {
    username: &'a str,
    token: &'a str,
}

impl GithubHandler<'_> {
    fn generate_repo_list(&self, page_no: u32) -> reqwest::RequestBuilder {
        let repo_url = format!("https://api.github.com/users/{}/repos", self.username);
        Client::new()
            .get(&repo_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", self.username)
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("per_page", "100"),
                ("page", page_no.to_string().as_str()),
            ])
            .basic_auth(self.username, Some(self.token.clone()))
    }

    fn generate_pulls_list(&self, repo: &Repository<'_>, page_no: u32) -> reqwest::RequestBuilder {
        let pull_url = repo.pulls_url.replace("{/number}", "");
        Client::new()
            .get(&pull_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", self.username)
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("state", "open"),
                ("per_page", "100"),
                ("page", page_no.to_string().as_str()),
            ])
            .basic_auth(self.username, Some(self.token.clone()))
    }

    fn generate_merge_pr(&self, repo: &Repository<'_>, pull: &Pull<'_>) -> reqwest::RequestBuilder {
        let merge_url = format!("{}/{}/merge", repo.pulls_url, pull.number);

        Client::new()
            .put(&merge_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", self.username)
            .basic_auth(self.username, Some(self.token.clone()))
    }
}

async fn merge_pr(
    repo: &Repository<'_>,
    pull: &Pull<'_>,
    gh_handler: &GithubHandler<'_>,
) -> Result<(), reqwest::Error> {
    gh_handler.generate_merge_pr(repo, pull).send().await?;

    Ok(())
}

async fn detect_dependabot_prs(
    repo: &Repository<'_>,
    gh_handler: &GithubHandler<'_>,
) -> Result<(), reqwest::Error> {
    println!(
        "{}",
        Colour::Yellow
            .italic()
            .paint(format!("Detecting dependabot PRs for {}", repo.name))
    );

    let mut page_no: u32 = 1;
    let depdenda_re: Regex = Regex::new(r"^Bump.*from.*to.*$").unwrap();
    loop {
        let response_text = gh_handler
            .generate_pulls_list(repo, page_no)
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
            merge_pr(&repo, &pull, &gh_handler).await?;
        }

        page_no += 1;
    }

    Ok(())
}

async fn iterate_repos(gh_handler: &GithubHandler<'_>) -> Result<(), reqwest::Error> {
    println!(
        "\n{}\n",
        Colour::Blue.paint("Fetching repositories for the user")
    );

    let mut page_no: u32 = 1;
    // TODO: Convert this to iterator and chain it to the
    // pull request iterator in the second place.
    loop {
        let response_text = gh_handler
            .generate_repo_list(page_no)
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

        join_all(
            repo_list
                .iter()
                .map(|repo| detect_dependabot_prs(repo, &gh_handler)),
        )
        .await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let username: String = std::env::args()
        .nth(1)
        .expect("Github username needs to be passed");

    let gh_handler: GithubHandler = GithubHandler {
        username: username.as_str(),
        token: GITHUB_TOKEN,
    };

    println!(
        "{}",
        Colour::Cyan
            .bold()
            .paint("Clean your dependabot PRs in fastest way")
    );

    iterate_repos(&gh_handler).await.unwrap()
}
