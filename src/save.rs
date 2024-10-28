use std::path::PathBuf;

use anyhow::Result;
use tokio::fs;

use crate::util::{explorer_open, get_file_update_time};

#[derive(Debug, Clone)]
pub struct Save {
	pub name: String,
	pub path: PathBuf,
	pub update_time: String,
	pub parent: PathBuf,
}

pub async fn read_saves() -> Result<Vec<Save>> {
	let mut saves = vec![];

	let saves_path = get_saves_path();

	let mut dirs = fs::read_dir(&saves_path).await?;

	while let Some(dir) = dirs.next_entry().await? {
		let file_type = dir.file_type().await?;
		if !file_type.is_dir() {
			continue;
		}
		let deeper_saves = read_depper_saves(dir.path()).await?;
		if !deeper_saves.is_empty() {
			saves.extend(deeper_saves);
		}
	}

	Ok(saves)
}

async fn read_depper_saves(parent: PathBuf) -> Result<Vec<Save>> {
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
fn get_saves_path() -> PathBuf {
	let mut cwd = std::env::current_dir().unwrap();

	cwd.push("tmp");
	cwd.push("saves");

	cwd
}

#[cfg(not(feature = "_dev"))]
fn get_saves_path() -> PathBuf {
	unimplemented!();
}

pub async fn open_saves_dir() {
	explorer_open(get_saves_path()).await;
}
