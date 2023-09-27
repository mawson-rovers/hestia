use std::error::Error;
use std::{env, fs};
use std::path::Path;
use log::info;
use uts_ws1::payload::Config;

fn main() {
    let dry_run = env::args().any(|a| a == "-n" || a == "-d");
    let config = Config::read();
    match config.install_path {
        None => panic!("Set UTS_INSTALL_PATH to installation path for self-update"),
        Some(install_path) => {
            let install_path = Path::new(&install_path);
            assert!(install_path.exists(), "UTS_INSTALL_PATH does not exist: {}",
                    install_path.display());
            update(install_path, dry_run).unwrap()
        }
    }
}

fn update(install_path: &Path, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("mawson-rovers")
        .repo_name("hestia")
        .build()?
        .fetch()?;

    // get the first available release
    let asset = releases[0].asset_for(self_update::get_target(), None).unwrap();
    info!("Downloading release from GitHub: {:?}", asset);

    let tmp_dir = tempfile::Builder::new()
        .prefix("self_update")
        .tempdir_in(std::env::current_dir()?)?;
    let tmp_tarball_path = tmp_dir.path().join(&asset.name);
    let tmp_tarball = std::fs::File::create(&tmp_tarball_path)?;

    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_tarball)?;
    tmp_tarball.sync_all()?;
    println!("Download completed, file size: {}", tmp_tarball.metadata()?.len());

    let dry_run_path = install_path.join("update_test");
    let path = if dry_run {
        fs::create_dir_all(&dry_run_path)?;
        dry_run_path.as_path()
    } else {
        install_path
    };
    info!("Unpacking update into: {}", path.display());
    self_update::Extract::from_source(&tmp_tarball_path)
        .archive(self_update::ArchiveKind::Tar(Some(self_update::Compression::Gz)))
        .extract_into(path)?;

    info!("Self update completed");

    Ok(())
}