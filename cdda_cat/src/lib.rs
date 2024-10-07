use crate::infra::archive::unpacker::{ArchiveAsset, ArchiveUnpacker};
use crate::infra::ui::cli::download_archive::download_archive;
use anyhow::{anyhow, Context, Error};
use app_dirs2::{app_root, AppDataType, AppInfo};
use bpaf::{construct, long, OptionParser, Parser};
use cdda_cat_data::entities::*;
use cdda_cat_lib::github_client::GithubClient;
use cdda_cat_lib::installation_manager::{AppSettings, CDDARelease};
use std::fs::{self, create_dir_all};
use std::path::Path;
use std::{default, process};
pub mod infra;

fn create_settings_file_unless_exists(settings_filepath: &Path) -> Result<(), Error> {
    if !Path::new(settings_filepath).exists() {
        AppSettings::default().write_to_file(settings_filepath)?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Options {
    Launch {
        release_tag: String,
        edition: Edition,
    },
    Install {
        release_tag: String,
        download_only: bool,
        overwrite: bool,
        edition: Option<Edition>,
    },
    Uninstall {
        release_tag: String,
        edition: Edition,
        remove_cdda_dir_only: bool,
    },
}

fn launch() -> impl Parser<Options> {
    let release_tag = long("tag").help("Tag name").argument("TAG");
    let edition = long("edition").help("Edition").argument("EDITION");
    construct!(Options::Launch {
        release_tag,
        edition
    })
}

fn install() -> impl Parser<Options> {
    let release_tag = long("tag").help("Tag name").argument("TAG");
    let download_only = long("download_only").help("Download only").switch();
    let overwrite = long("overwrite").help("Overwrite").switch();
    let edition = long("edition")
        .help("Edition")
        .argument("EDITION")
        .optional();
    construct!(Options::Install {
        release_tag,
        download_only,
        overwrite,
        edition
    })
}

fn uninstall() -> impl Parser<Options> {
    let release_tag = long("tag").help("Tag name").argument("TAG");
    let edition = long("edition").help("Edition").argument("EDITION");
    let remove_cdda_dir_only = long("remove_cdda_dir_only").help("REMOVE DIR").switch();

    construct!(Options::Uninstall {
        release_tag,
        edition,
        remove_cdda_dir_only,
    })
}

fn options() -> OptionParser<Options> {
    let launch = launch()
        .to_options()
        .descr("Launch a game")
        .command("launch");
    let install = install().to_options().descr("Install").command("install");
    let uninstall = uninstall()
        .to_options()
        .descr("Uninstall")
        .command("uninstall");

    construct!([launch, install, uninstall]).to_options()
}
const APP_INFO: AppInfo = AppInfo {
    name: "cdda_cat",
    author: "cloudyluna",
};

pub async fn run() -> anyhow::Result<()> {
    let system_config_path = app_root(AppDataType::UserConfig, &APP_INFO)?;
    let system_cache_path = app_root(AppDataType::UserCache, &APP_INFO)?;
    let settings_file_path = system_config_path
        .join("settings.json")
        .as_path()
        .to_owned();
    create_settings_file_unless_exists(&settings_file_path)?;
    let mut settings = AppSettings::default().read_from_file(&settings_file_path)?;

    let gh_client = GithubClient::new(
        &settings.upstream_repository.owner_name,
        &settings.upstream_repository.repository_name,
    );

    match options().run() {
        Options::Launch {
            release_tag,
            edition,
        } => {
            let asset = settings
                .installed_games
                .iter()
                .find(|asset| {
                    asset.platform == Platform::Linux
                        && asset.tag == release_tag
                        && asset.edition == edition
                })
                .with_context(|| {
                    format!(
                        "No asset with release tag of {} and edition of {} was found to uninstall!",
                        release_tag, edition
                    )
                })?;
            println!("Found existing installation!");
            let launcher_path = asset
                .game_edition_directory_path
                .join(settings.decompressed_game_directory_path.as_path())
                .join(settings.launcher_name.as_str());
            println!("Running {}", &launcher_path.display());
            process::Command::new(launcher_path)
                .spawn()
                .with_context(|| format!("Failed to launch {}", settings.launcher_name.as_str()))?
                .wait()?;
        }
        Options::Install {
            release_tag,
            download_only,
            overwrite,
            edition,
        } => {
            let system_download_dir = system_cache_path
                .join(settings.root_download_directory_path.as_path())
                .into_os_string()
                .into_string()
                .map_err(|_| anyhow!("Cannot convert top download directory path to String"))?;
            let root_download_directory_path = RootDownloadDirectoryPath::new(&system_download_dir);
            let release = CDDARelease::fetch_by_tag(gh_client, &release_tag).await?;
            let mut release_assets = release.assets.iter().map(|asset| Asset {
                name: asset.name.to_string(),
                tag: release.tag_name.to_string(),
                platform: Platform::from(asset.name.as_str()),
                edition: Edition::from(asset.name.as_str()),
                url: asset.browser_download_url.to_string(),
                game_edition_directory_path: GameEditionDirectoryPath::default(),
            });

            let asset = release_assets
                .find(|a: &Asset| -> bool {
                    if let Some(ed) = &edition {
                        a.edition == *ed
                    } else {
                        a.edition == Edition::default()
                    }
                })
                .with_context(|| {
                    format!(
                        "No asset with release tag of {} and edition of {} was found to uninstall!",
                        release_tag,
                        edition.unwrap()
                    )
                })?;

            let game_edition_directory_path =
                root_download_directory_path.to_game_edition_directory_path(&asset);
            let is_game_directory_exists = game_edition_directory_path
                .join(settings.decompressed_game_directory_path.as_path())
                .exists();
            if is_game_directory_exists && !overwrite {
                eprintln!(
                    "{} installation directory already exists and we won't overwrite it!",
                    settings.decompressed_game_directory_path.display()
                );
                eprintln!("Retry with --overwrite flag to force installation directory overwrite.");
                eprintln!("Aborted for now");

                process::exit(1);
            }

            create_dir_all(game_edition_directory_path.as_path())?;
            let archive_path = ArchiveFilePath::new(game_edition_directory_path.join(&asset.name));

            download_archive(&asset.url, &archive_path).await?;

            if !download_only {
                ArchiveAsset::new(asset).unpack(
                    &mut settings,
                    &archive_path,
                    &game_edition_directory_path,
                    &settings_file_path,
                )?;
            }
            println!("\nDone!")
        }
        Options::Uninstall {
            release_tag,
            edition,
            remove_cdda_dir_only,
        } => {
            let asset = settings
                .installed_games
                .iter()
                // TODO: Check OS and match platform type.
                .find(|asset| asset.tag == release_tag && asset.edition == edition)
                .with_context(|| {
                    format!(
                        "No asset with release tag of {} and edition of {} was found to uninstall!",
                        release_tag, edition
                    )
                })?;

            println!("Found existing installation!");
            println!("Uninstalling..");
            let game_edition_directory_path = &asset.game_edition_directory_path;
            let decompressed_game_directory_path = &game_edition_directory_path
                .as_path()
                .join(settings.decompressed_game_directory_path.as_path());

            if remove_cdda_dir_only {
                fs::remove_dir_all(decompressed_game_directory_path).with_context(|| {
                    format!(
                        "Failed to remove CDDA's installation directory at: {}",
                        decompressed_game_directory_path.display()
                    )
                })?;
            } else {
                fs::remove_dir_all(game_edition_directory_path.as_path()).with_context(|| {
                    format!(
                        "Failed to remove installation directory path of: {}",
                        &game_edition_directory_path.display()
                    )
                })?;
            }
            let new_assets: Vec<Asset> = settings
                .installed_games
                .iter()
                .filter(|x| *x != asset)
                .cloned()
                .collect();
            settings.installed_games = ReleaseAssets::new(new_assets);
            settings.write_to_file(&settings_file_path)?;
            println!("Finished uninstall!");
        }
    }

    Ok(())
}
