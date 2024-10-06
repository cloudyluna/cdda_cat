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
#[async_trait]
impl RepositoryReleaseClient for GithubClient {
    async fn get_by_tag(&self, tag: &str) -> Result<Release, Error> {
        Ok(reqwest::get(format!(
            "{}/{}/{}/releases/tags/{}",
            API_ROOT, self.owner_name, self.repo_name, tag
        ))
        .await?
        .json::<Release>()
        .await?)
    }
}
