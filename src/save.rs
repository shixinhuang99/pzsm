use std::path::PathBuf;

use anyhow::Result;
use tokio::fs;

#[cfg(not(feature = "_dev"))]
use crate::util::home_dir;
use crate::util::{explorer_open, file_exists, get_file_update_time};

#[derive(Debug, Clone)]
pub struct Save {
	pub name: String,
	pub path: PathBuf,
	pub update_time: String,
	pub parent: PathBuf,
}

pub async fn read_saves() -> Result<Vec<Save>> {
	let mut saves = vec![];

	let saves_path = get_saves_path()?;

	let mut dirs = fs::read_dir(&saves_path).await?;

	while let Some(dir) = dirs.next_entry().await? {
		let file_type = dir.file_type().await?;
		if !file_type.is_dir() {
			continue;
		}
		let deeper_saves = read_deeper_saves(dir.path()).await?;
		if !deeper_saves.is_empty() {
			saves.extend(deeper_saves);
		}
	}

	Ok(saves)
}

async fn read_deeper_saves(parent: PathBuf) -> Result<Vec<Save>> {
	let mut saves = vec![];

	let mut dirs = fs::read_dir(&parent).await?;

	while let Some(dir) = dirs.next_entry().await? {
		let file_type = dir.file_type().await?;
		if !file_type.is_dir() {
			continue;
		}
		let name = dir.file_name().to_string_lossy().to_string();
		let path = dir.path();
		let update_time = get_file_update_time(&path).await?;
		saves.push(Save {
			name,
			path,
			update_time,
			parent: parent.clone(),
		})
	}

	Ok(saves)
}

#[cfg(feature = "_dev")]
fn get_saves_path() -> Result<PathBuf> {
	let mut cwd = std::env::current_dir()?;

	cwd.push("tmp");
	cwd.push("saves");

	Ok(cwd)
}

#[cfg(not(feature = "_dev"))]
fn get_saves_path() -> Result<PathBuf> {
	let mut home_dir = home_dir()?;

	home_dir.push("Zomboid");
	home_dir.push("saves");

	Ok(home_dir)
}

pub async fn open_saves_dir() {
	let Ok(saves_path) = get_saves_path() else {
		return;
	};
	if !file_exists(&saves_path).await {
		return;
	}
	explorer_open(saves_path).await;
}
