use anyhow::Error;
use cdda_cat_data::entities::{ArchiveFilePath, Asset, GameEditionDirectoryPath};
use cdda_cat_lib::installation_manager::AppSettings;
use derive_more::{Deref, DerefMut};
use std::path::Path;

#[derive(Debug, PartialEq, Deref, DerefMut)]
pub struct ArchiveAsset(Asset);

impl ArchiveAsset {
    pub fn new(asset: Asset) -> Self {
        Self(asset)
    }
}

pub trait ArchiveUnpacker {
    fn unpack(
        &mut self,
        settings: &mut AppSettings,
        archive_file_path: &ArchiveFilePath,
        game_edition_directory_path: &GameEditionDirectoryPath,
        settings_file_path: &Path,
    ) -> Result<(), Error>;
}
