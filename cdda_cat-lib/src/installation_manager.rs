use crate::github_client::{GithubClient, RepositoryReleaseClient};
use anyhow::{Context, Error};
use cdda_cat_data::entities::{DateTimePublished, Release, Settings};
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, PartialEq, Default, Deref)]
pub struct CDDARelease(Release);

impl CDDARelease {
    pub async fn fetch_by_tag(client: GithubClient, tag: &str) -> Result<CDDARelease, Error> {
        CDDARelease::try_from(client.get_by_tag(tag).await?)
    }
}

impl TryFrom<Release> for CDDARelease {
    type Error = anyhow::Error;
    fn try_from(release: Release) -> Result<CDDARelease, Self::Error> {
        let release = Release {
            name: release.name,
            tag_name: release.tag_name.to_string(),
            body: release.body,
            published_at: DateTimePublished::new(*release.published_at),
            url: release.url.to_string(),
            assets: release.assets,
        };

        Ok(CDDARelease(release))
    }
}

#[derive(Debug, PartialEq, Default, Deref, DerefMut, Serialize, Deserialize, Clone)]
pub struct AppSettings(Settings);

impl AppSettings {
    pub fn read_from_file(&self, settings_filepath: &Path) -> Result<AppSettings, Error> {
        fs::read_to_string(settings_filepath)
            .and_then(|x| Ok(serde_json::from_str::<AppSettings>(&x)?))
            .with_context(|| format!("Failed to read from {} file!", settings_filepath.display()))
    }

    pub fn write_to_file(&self, settings_filepath: &Path) -> Result<(), Error> {
        fs::write(settings_filepath, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("Failed to write to {} file!", settings_filepath.display()))
    }
}
