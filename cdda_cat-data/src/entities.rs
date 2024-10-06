use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut, Display};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Clone, Display, Serialize, Deserialize)]
pub enum Platform {
    #[default]
    #[display(fmt = "linux")]
    Linux,
    #[display(fmt = "unsupported-platform")]
    Unsupported,
}

impl ::core::str::FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Platform::from(s))
    }
}

impl From<&str> for Platform {
    fn from(archive_name: &str) -> Self {
        if archive_name.to_lowercase().contains("linux") {
            Platform::Linux
        } else {
            Platform::Unsupported
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Display, Serialize, Deserialize)]
pub enum TilesEdition {
    #[default]
    #[display(fmt = "with-sound-pack")]
    /// With CC-Sounds sound pack
    WithSoundPack,
    #[display(fmt = "without-sound-pack")]
    /// Regular edition without any bundled sound pack
    WithoutSoundPack,
}

impl From<&str> for TilesEdition {
    fn from(archive_name: &str) -> Self {
        if archive_name.to_lowercase().contains("sounds") {
            TilesEdition::WithSoundPack
        } else {
            TilesEdition::WithoutSoundPack
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Display, Serialize, Deserialize)]
pub enum Edition {
    #[default]
    #[display(fmt = "curses")]
    /// Non-GUI edition
    Curses,
    #[display(fmt = "tiles-{}", _0)]
    /// GUI edition
    Tiles(TilesEdition),
}

impl ::core::str::FromStr for TilesEdition {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.to_lowercase() == "with-sound-pack" {
            TilesEdition::WithSoundPack
        } else {
            TilesEdition::WithoutSoundPack
        })
    }
}

impl ::core::str::FromStr for Edition {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.to_lowercase() == "curses" {
            Edition::Curses
        } else {
            Edition::Tiles(TilesEdition::from_str(s.to_lowercase().as_str())?)
        })
    }
}

impl From<&str> for Edition {
    fn from(archive_name: &str) -> Self {
        let lowercase_name = archive_name.to_lowercase();
        if lowercase_name.contains("curses") {
            Edition::Curses
        } else {
            Edition::Tiles(TilesEdition::from(lowercase_name.as_str()))
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize, Deref)]
pub struct ArchiveFilePath(PathBuf);

impl ArchiveFilePath {
    pub fn new(path: PathBuf) -> ArchiveFilePath {
        Self(path)
    }
}

impl ::core::str::FromStr for ArchiveFilePath {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArchiveFilePath(Path::new(s).to_path_buf()))
    }
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub tag: String,
    pub platform: Platform,
    pub edition: Edition,
    pub url: String,
    pub game_edition_directory_path: GameEditionDirectoryPath,
}

#[derive(Debug, PartialEq, Default, Deref, Serialize, Deserialize)]
pub struct DateTimePublished(Option<DateTime<Utc>>);

impl DateTimePublished {
    pub fn new(datetime: Option<DateTime<Utc>>) -> Self {
        Self(datetime)
    }
}

#[derive(Debug, PartialEq, Default, Deref, DerefMut, Serialize, Deserialize, Clone)]
pub struct ReleaseAssets(Vec<Asset>);

impl ReleaseAssets {
    pub fn new(assets: Vec<Asset>) -> Self {
        Self(assets)
    }
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Release {
    pub name: String,
    pub tag: String,
    pub description: String,
    pub published_at: DateTimePublished,
    pub url: String,
    pub assets: ReleaseAssets,
}

impl Release {
    pub fn get_asset(&self, platform: Platform, edition: Edition) -> Option<Asset> {
        self.assets
            .iter()
            .find(|asset| asset.platform == platform && asset.edition == edition)
            .cloned()
    }
}

#[derive(Debug, PartialEq, Deref, Serialize, Deserialize, Clone)]
pub struct RootDownloadDirectoryPath(PathBuf);

impl RootDownloadDirectoryPath {
    pub fn new(path: &str) -> Self {
        RootDownloadDirectoryPath(Path::new(path).to_path_buf())
    }

    pub fn to_game_edition_directory_path(&self, asset: &Asset) -> GameEditionDirectoryPath {
        GameEditionDirectoryPath(
            self.join(asset.platform.to_string())
                .join(&asset.tag)
                .join(asset.edition.to_string()),
        )
    }
}

impl Default for RootDownloadDirectoryPath {
    fn default() -> Self {
        Self::new("cdda-downloads")
    }
}

#[derive(Debug, PartialEq, Deref, Serialize, Deserialize, Clone, Default)]
pub struct GameEditionDirectoryPath(PathBuf);

impl ::core::str::FromStr for GameEditionDirectoryPath {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GameEditionDirectoryPath(Path::new(s).to_path_buf()))
    }
}

#[derive(Debug, PartialEq, Deref, Serialize, Deserialize, Clone)]
pub struct DecompressedGameDirectoryPath(PathBuf);

impl DecompressedGameDirectoryPath {
    pub fn new(path: &str) -> Self {
        DecompressedGameDirectoryPath(Path::new(path).to_path_buf())
    }
}

impl ::core::str::FromStr for DecompressedGameDirectoryPath {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DecompressedGameDirectoryPath::new(s))
    }
}

impl Default for DecompressedGameDirectoryPath {
    fn default() -> Self {
        Self::new("CDDA")
    }
}

#[derive(Debug, PartialEq, Deref, Serialize, Deserialize, Clone)]
pub struct LauncherName(String);

impl LauncherName {
    pub fn new(name: &str) -> Self {
        LauncherName(name.to_string())
    }
}

impl ::core::str::FromStr for LauncherName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LauncherName::new(s))
    }
}

impl Default for LauncherName {
    fn default() -> Self {
        Self::new("cataclysm-launcher")
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UpstreamRepository {
    pub owner_name: String,
    pub repository_name: String,
}

impl Default for UpstreamRepository {
    fn default() -> Self {
        Self {
            owner_name: "CleverRaven".to_string(),
            repository_name: "Cataclysm-DDA".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub upstream_repository: UpstreamRepository,
    pub root_download_directory_path: RootDownloadDirectoryPath,
    pub decompressed_game_directory_path: DecompressedGameDirectoryPath,
    pub launcher_name: LauncherName,
    pub installed_games: ReleaseAssets,
}

impl Settings {
    pub fn new(
        target_repository: UpstreamRepository,
        root_download_directory: &str,
        installation_directory: &str,
        launcher_name: &str,
        installed_games: Vec<Asset>,
    ) -> Self {
        Self {
            upstream_repository: target_repository,
            root_download_directory_path: RootDownloadDirectoryPath::new(root_download_directory),
            decompressed_game_directory_path: DecompressedGameDirectoryPath::new(
                installation_directory,
            ),
            launcher_name: LauncherName::new(launcher_name),
            installed_games: ReleaseAssets::new(installed_games),
        }
    }
}
