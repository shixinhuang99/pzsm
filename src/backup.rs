use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use tokio::fs;

use crate::{
	save::Save,
	util::{
		copy_dir, explorer_open, file_exists, move_to_recycle_bin, time_now,
	},
};

pub type BackupMap = HashMap<String, Vec<Backup>>;

#[derive(Debug, Clone)]
pub struct Backup {
	pub name: String,
	pub path: PathBuf,
}

pub async fn read_backups() -> Result<BackupMap> {
	let path = get_backups_path();

	if !file_exists(&path).await {
		fs::create_dir(&path).await?;
	}

	let mut dirs = fs::read_dir(&path).await?;
	let mut backup_map = HashMap::new();

	while let Some(dir) = dirs.next_entry().await? {
		let file_type = dir.file_type().await?;
		if !file_type.is_dir() {
			continue;
		}
		let backups = read_deeper_backup(dir.path()).await?;
		if !backups.is_empty() {
			let name = dir.file_name().to_string_lossy().to_string();
			backup_map.insert(name, backups);
		}
	}

	Ok(backup_map)
}

async fn read_deeper_backup(path: PathBuf) -> Result<Vec<Backup>> {
	let mut dirs = fs::read_dir(&path).await?;
	let mut backups = vec![];

	while let Some(dir) = dirs.next_entry().await? {
		let file_type = dir.file_type().await?;
		if !file_type.is_dir() {
			continue;
		}
		backups.push(Backup {
			name: dir.file_name().to_string_lossy().to_string(),
			path: dir.path(),
		});
	}

	Ok(backups)
}

#[cfg(feature = "_dev")]
fn get_backups_path() -> PathBuf {
	let mut cwd = std::env::current_dir().unwrap();

	cwd.push("tmp");
	cwd.push("pzsm_backup");

	cwd
}

#[cfg(not(feature = "_dev"))]
fn get_backups_path() -> PathBuf {
	unimplemented!();
}

pub async fn create_backup(save: Save) -> Result<()> {
	let name = time_now();
	let mut path = get_backups_path();

	path.push(&save.name);
	path.push(&name);
	path.push(&save.name);

	if !file_exists(&path).await {
		fs::create_dir_all(&path).await?;
	}

	copy_dir(&save.path, &path).await?;

	Ok(())
}

pub async fn apply_backup(save: Save, backup_path: PathBuf) -> Result<()> {
	let tmp_path = {
		let mut p = save.path.clone();
		p.pop();
		p.push(format!("{}_tmp", save.name));
		p
	};

	fs::rename(&save.path, &tmp_path).await?;
	copy_dir(&backup_path, &save.parent).await?;
	move_to_recycle_bin(&tmp_path).await?;

	Ok(())
}

pub async fn delete_backup(backup_path: PathBuf) -> Result<()> {
	move_to_recycle_bin(&backup_path).await?;
	Ok(())
}

pub async fn delete_backup_many(paths: Vec<PathBuf>) -> Result<()> {
	for path in paths {
		move_to_recycle_bin(&path).await?;
	}
	Ok(())
}

pub async fn open_backups_dir() {
	explorer_open(get_backups_path()).await;
}
