use anyhow::Context;
use anyhow::Error;
use cdda_cat_data::entities::ArchiveFilePath;
use cdda_cat_data::entities::GameEditionDirectoryPath;
use cdda_cat_lib::installation_manager::AppSettings;
use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::cmp::min;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use super::unpacker::ArchiveAsset;
use super::unpacker::ArchiveUnpacker;

fn read_archive_file(archive_file_path: &ArchiveFilePath) -> Result<File, Error> {
    File::open(archive_file_path.display().to_string()).with_context(|| {
        format!(
            "Failed to read archive file: {}",
            archive_file_path.display()
        )
    })
}

impl ArchiveUnpacker for ArchiveAsset {
    fn unpack(
        &mut self,
        settings: &mut AppSettings,
        archive_file_path: &ArchiveFilePath,
        game_edition_directory_path: &GameEditionDirectoryPath,
        settings_filepath: &Path,
    ) -> Result<(), Error> {
        println!("Starting to unpack archive..");

        // Not sure why we have to read the file twice to prevent
        // read from 0 runtime error.
        let file_for_size = read_archive_file(archive_file_path)?;
        let total_size = tar::Archive::new(GzDecoder::new(&file_for_size))
            .entries()?
            .count() as u64;
        let file = read_archive_file(archive_file_path)?;
        let mut archive = tar::Archive::new(GzDecoder::new(&file));

        let progress_bar = ProgressBar::new(total_size);
        progress_bar.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})")?
        .progress_chars("#>-"));
        progress_bar.set_message("Extracting archive entries..");
        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> Result<PathBuf, anyhow::Error> {
                let _entry = entry.path()?;
                let mut components = _entry.components();
                components.next();
                let removed_old_prefix_path = components.as_path();

                let new_path = game_edition_directory_path
                    .join(settings.decompressed_game_directory_path.as_path())
                    .join(removed_old_prefix_path);
                entry.unpack(&new_path)?;
                Ok(new_path)
            })
            .filter_map(|e| e.ok())
            .enumerate()
            .for_each(|(i, _)| {
                progress_bar.set_position(min(i as u64 + 1, total_size));
            });

        progress_bar.set_message("Finished unpacking!");

        self.game_edition_directory_path = game_edition_directory_path.to_owned();
        
        if !settings.installed_games.contains(self) {
            settings.installed_games.push(self.to_owned());
            settings.write_to_file(settings_filepath)?;
        }

        Ok(())
    }
}
