use std::{
	path::{Path, PathBuf},
	process::Command,
	time::SystemTime,
};

use anyhow::Result;
use chrono::{DateTime, Local};
use tokio::fs;

use crate::consts::APP_REPO;

fn format_systime(time: SystemTime) -> String {
	let dt: DateTime<Local> = DateTime::from(time);
	dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn time_now() -> String {
	let dt = Local::now();
	dt.format("%Y_%m_%d_%H_%M_%S").to_string()
}

pub async fn file_exists(p: &Path) -> bool {
	fs::try_exists(p).await.is_ok_and(|v| v)
}

pub async fn get_file_update_time(p: &Path) -> Result<String> {
	let metadata = fs::metadata(&p).await?;
	let modified = metadata.modified()?;

	Ok(format_systime(modified))
}

pub async fn copy_dir(source: &Path, dest: &Path) -> Result<()> {
	let source = source.to_path_buf();
	let dest = dest.to_path_buf();

	tokio::task::spawn_blocking(move || -> Result<()> {
		dircpy::copy_dir(source, dest)?;
		Ok(())
	})
	.await?
}

pub async fn move_to_recycle_bin(path: &Path) -> Result<()> {
	let path = path.to_path_buf();

	tokio::task::spawn_blocking(move || -> Result<()> {
		trash::delete(path)?;
		Ok(())
	})
	.await?
}

macro_rules! command {
	($name:literal, $args:tt) => {
		if let Err(err) = tokio::task::spawn_blocking(|| {
			if let Err(err) = Command::new($name).args($args).output() {
				eprintln!("{}", err);
			};
		})
		.await
		{
			eprintln!("{}", err);
		}
	};
}

pub async fn explorer_open(path: PathBuf) {
	command!("explorer", [path]);
}

pub async fn open_github_page() {
	command!("cmd", ["/c", "start", APP_REPO]);
}

#[cfg(not(feature = "_dev"))]
pub fn home_dir() -> Result<PathBuf> {
	home::home_dir().ok_or(anyhow::anyhow!("Unable to get your home dir"))
}
