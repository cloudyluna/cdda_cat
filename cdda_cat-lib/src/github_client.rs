use anyhow::Error;
use async_trait::async_trait;
use cdda_cat_data::entities::Release;

#[async_trait]
pub trait RepositoryReleaseClient {
    async fn get_by_tag(&self, tag: &str) -> Result<Release, Error>;
}

pub struct GithubClient {
    owner_name: String,
    repo_name: String,
}

const API_ROOT: &str = "https://api.github.com/repos";
impl GithubClient {
    pub fn new(owner_name: &str, repo_name: &str) -> Self {
        Self {
            owner_name: owner_name.to_string(),
            repo_name: repo_name.to_string(),
        }
    }
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[async_trait]
impl RepositoryReleaseClient for GithubClient {
    async fn get_by_tag(&self, tag: &str) -> Result<Release, Error> {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;

        Ok(client
            .get(format!(
                "{}/{}/{}/releases/tags/{}",
                API_ROOT, self.owner_name, self.repo_name, tag
            ))
            .send()
            .await?
            .json::<Release>()
            .await?)
    }
}
