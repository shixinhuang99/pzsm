use std::{collections::HashMap, path::PathBuf};

use iced::{
	alignment,
	widget::{
		button, column, container, row, scrollable, text, vertical_space,
	},
	Element, Length, Padding, Task, Theme,
};

use crate::{
	backup::{
		apply_backup, create_backup, delete_backup, delete_backup_many,
		open_backups_dir, read_backups, Backup, BackupMap,
	},
	component::{
		backup_button, body_centered_text, horizontal_divider, panel,
		panel_centered_text, row_between, save_button, vertical_divider,
	},
	consts::{APP_NAME, APP_VER},
	save::{open_saves_dir, read_saves, Save},
	util::open_github_page,
};

macro_rules! error_handler {
	($msg:path) => {
		|ret| match ret {
			Ok(_) => $msg,
			Err(err) => Message::Err(err.to_string()),
		}
	};
	($msg:path => ()) => {
		|ret| match ret {
			Ok(v) => $msg(v),
			Err(err) => Message::Err(err.to_string()),
		}
	};
}

#[derive(Default)]
struct Pzsm {
	saves: Vec<Save>,
	selected_save: Option<Save>,
	backup_map: HashMap<String, Vec<Backup>>,
	selected_backup: Option<Backup>,
	body_mask: String,
	error_mask: String,
}

#[derive(Debug, Clone)]
enum Message {
	Err(String),
	ReadSaves,
	ReadSavesOk(Vec<Save>),
	ReadBackups,
	ReadBackupsOk(HashMap<String, Vec<Backup>>),
	SaveSelected(Save),
	BackupSelected(Backup),
	NewBackup,
	ApplyBackup,
	DeleteBackup,
	DeleteUnusedBackups,
	OpenSavesDir,
	OpenBackupsDir,
	OpenGithubPage,
}

impl Pzsm {
	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::Err(err) => {
				self.handle_err(err);
			}
			Message::ReadSaves => {
				return self.handle_read_saves();
			}
			Message::ReadSavesOk(saves) => {
				return self.handle_read_saves_ok(saves);
			}
			Message::ReadBackups => {
				return self.handle_read_backups();
			}
			Message::ReadBackupsOk(backup_map) => {
				self.handle_read_backups_ok(backup_map);
			}
			Message::SaveSelected(save) => {
				self.handle_save_selected(save);
			}
			Message::NewBackup => {
				return self.handle_new_backup();
			}
			Message::BackupSelected(backup) => {
				self.handle_backup_selected(backup);
			}
			Message::ApplyBackup => {
				return self.handle_apply_backup();
			}
			Message::DeleteBackup => {
				return self.handle_delete_backup();
			}
			Message::DeleteUnusedBackups => {
				return self.handle_delete_unused_backups();
			}
			Message::OpenSavesDir => {
				return self.handle_open_saves_dir();
			}
			Message::OpenBackupsDir => {
				return self.handle_open_backups_dir();
			}
			Message::OpenGithubPage => {
				return self.handle_open_github_page();
			}
		}

		Task::none()
	}

	fn theme(&self) -> Theme {
		Theme::TokyoNight
	}

	fn view(&self) -> Element<Message> {
		let header = self.header();
		let body = self.body();
		container(column([header, horizontal_divider().into(), body])).into()
	}

	fn header(&self) -> Element<Message> {
		let title = text("Pzsm").size(20);
		let ver = text(APP_VER).size(14);
		let open_github_page_btn = button(text("Goto GitHub page").size(12))
			.on_press(Message::OpenGithubPage);
		let left_row =
			row([title.into(), ver.into(), open_github_page_btn.into()])
				.align_y(alignment::Vertical::Bottom)
				.spacing(12);

		let refresh_btn =
			button("Refresh").on_press_maybe(if self.body_mask.is_empty() {
				Some(Message::ReadSaves)
			} else {
				None
			});
		let delete_unused_backups_btn = button("Delete unused backups")
			.style(button::danger)
			.on_press_maybe(
				if self.error_mask.is_empty() && self.body_mask.is_empty() {
					Some(Message::DeleteUnusedBackups)
				} else {
					None
				},
			);
		let open_saves_dir_btn =
			button("Open saves dir").on_press(Message::OpenSavesDir);
		let open_backups_dir_btn =
			button("Open backups dir").on_press(Message::OpenBackupsDir);
		let right_row = row([
			refresh_btn.into(),
			delete_unused_backups_btn.into(),
			open_saves_dir_btn.into(),
			open_backups_dir_btn.into(),
		])
		.spacing(6);

		row_between(50, left_row, right_row).into()
	}

	fn body(&self) -> Element<Message> {
		if !self.error_mask.is_empty() {
			return body_centered_text(&self.error_mask).into();
		}

		if !self.body_mask.is_empty() {
			return body_centered_text(&self.body_mask).into();
		}

		let left_side = self.left_side();
		let right_side = self.right_side();
		row([left_side, vertical_divider().into(), right_side]).into()
	}

	fn left_side(&self) -> Element<Message> {
		if self.saves.is_empty() {
			return panel_centered_text("No saves").into();
		}

		let is_selected_save = |s: &str| -> bool {
			self.selected_save
				.as_ref()
				.is_some_and(|save| save.name == s)
		};

		let save_btns: Vec<Element<Message>> = self
			.saves
			.iter()
			.map(|save| {
				save_button(save, Message::SaveSelected, is_selected_save)
					.into()
			})
			.collect();

		panel(
			scrollable(column(save_btns).spacing(6))
				.spacing(6)
				.height(Length::Fill),
		)
		.into()
	}

	fn right_side(&self) -> Element<Message> {
		let Some(selected_save) = &self.selected_save else {
			return panel_centered_text("Please select a save").into();
		};

		let on_op_press_maybe = |msg: Message| -> Option<Message> {
			if self.selected_backup.is_some() {
				Some(msg)
			} else {
				None
			}
		};

		let operations = container(
			row([
				button("New backup").on_press(Message::NewBackup).into(),
				button("Apply")
					.on_press_maybe(on_op_press_maybe(Message::ApplyBackup))
					.into(),
				button("Delete")
					.on_press_maybe(on_op_press_maybe(Message::DeleteBackup))
					.style(button::danger)
					.into(),
			])
			.spacing(6),
		)
		.padding(Padding::new(12.0));

		let is_selected_backup = |s: &str| -> bool {
			self.selected_backup
				.as_ref()
				.is_some_and(|backup| backup.name == s)
		};

		let backup_content: Element<Message> = if let Some(backup_list) =
			self.backup_map.get(&selected_save.name)
		{
			scrollable(
				column(
					backup_list
						.iter()
						.rev()
						.map(|backup| {
							backup_button(
								backup,
								Message::BackupSelected,
								is_selected_backup,
							)
							.into()
						})
						.collect::<Vec<Element<Message>>>(),
				)
				.spacing(6),
			)
			.spacing(6)
			.height(Length::Fill)
			.into()
		} else {
			panel_centered_text("No backups").into()
		};

		panel(column([
			operations.into(),
			horizontal_divider().into(),
			vertical_space().height(8).into(),
			backup_content,
		]))
		.into()
	}

	fn handle_err(&mut self, err: String) {
		eprintln!("{}", err);
		self.error_mask = err;
	}

	fn handle_read_saves(&mut self) -> Task<Message> {
		self.body_mask = "Loading saves...".to_string();
		Task::perform(read_saves(), error_handler!(Message::ReadSavesOk => ()))
	}

	fn handle_read_saves_ok(&mut self, saves: Vec<Save>) -> Task<Message> {
		self.saves = saves;
		if self.selected_save.as_ref().is_some_and(|selected_save| {
			!self
				.saves
				.iter()
				.any(|save| selected_save.name == save.name)
		}) {
			self.selected_save = None;
		}
		Task::done(Message::ReadBackups)
	}

	fn handle_read_backups(&mut self) -> Task<Message> {
		self.body_mask = "Loading backups...".to_string();
		Task::perform(
			read_backups(),
			error_handler!(Message::ReadBackupsOk => ()),
		)
	}

	fn handle_read_backups_ok(&mut self, backup_map: BackupMap) {
		self.body_mask.clear();
		self.backup_map = backup_map;
		if let Some(save) = &self.selected_save {
			self.select_last_backup(save.name.clone());
		}
	}

	fn select_last_backup(&mut self, save_name: String) {
		if let Some(backup_list) = self.backup_map.get(&save_name) {
			if let Some(last) = backup_list.last() {
				self.selected_backup = Some(last.clone());
			}
		} else {
			self.selected_backup = None;
		}
	}

	fn handle_save_selected(&mut self, save: Save) {
		self.select_last_backup(save.name.clone());
		self.selected_save = Some(save);
	}

	fn handle_new_backup(&mut self) -> Task<Message> {
		if let Some(save) = &self.selected_save {
			self.body_mask = "Creating backup...".to_string();
			return Task::perform(
				create_backup(save.clone()),
				error_handler!(Message::ReadBackups),
			);
		}
		Task::none()
	}

	fn handle_backup_selected(&mut self, backup: Backup) {
		self.selected_backup = Some(backup);
	}

	fn handle_apply_backup(&mut self) -> Task<Message> {
		if let (Some(save), Some(backup)) =
			(&self.selected_save, &self.selected_backup)
		{
			self.body_mask = "Applying backup...".to_string();
			return Task::perform(
				apply_backup(save.clone(), backup.path.clone()),
				error_handler!(Message::ReadSaves),
			);
		}
		Task::none()
	}

	fn handle_delete_backup(&mut self) -> Task<Message> {
		if let Some(backup) = &self.selected_backup {
			self.body_mask = "Deleting backup...".to_string();
			return Task::perform(
				delete_backup(backup.path.clone()),
				error_handler!(Message::ReadBackups),
			);
		}
		Task::none()
	}

	fn handle_delete_unused_backups(&mut self) -> Task<Message> {
		let mut unused_backup_paths: Vec<PathBuf> = vec![];

		let is_unused =
			|name: &str| -> bool { !self.saves.iter().any(|s| s.name == name) };

		for (name, backups) in &self.backup_map {
			if is_unused(name) && !backups.is_empty() {
				if let Some(path) = backups[0].path.parent() {
					unused_backup_paths.push(path.to_path_buf());
				}
			}
		}

		if unused_backup_paths.is_empty() {
			return Task::none();
		}

		self.body_mask = "Deleting unused backups...".to_string();
		Task::perform(
			delete_backup_many(unused_backup_paths),
			error_handler!(Message::ReadSaves),
		)
	}

	fn handle_open_saves_dir(&self) -> Task<Message> {
		Task::future(open_saves_dir()).discard()
	}

	fn handle_open_backups_dir(&self) -> Task<Message> {
		Task::future(open_backups_dir()).discard()
	}

	fn handle_open_github_page(&self) -> Task<Message> {
		Task::future(open_github_page()).discard()
	}
}

pub fn run() -> iced::Result {
	use iced::{application, window, Font, Settings, Size};

	let size = Size::new(1000.0, 600.0);

	let mut app = application("Pzsm", Pzsm::update, Pzsm::view)
		.settings(Settings {
			id: Some(APP_NAME.to_string()),
			..Default::default()
		})
		.window(window::Settings {
			size,
			min_size: Some(size),
			position: window::Position::Centered,
			..Default::default()
		})
		.theme(Pzsm::theme);

	if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\simhei.ttf") {
		app = app.font(data).default_font(Font::with_name("黑体"));
	}

	app.run_with(|| {
		(
			Pzsm::default(),
			Task::batch([
				window::get_latest().and_then(|id| {
					if let Ok(icon) = window::icon::from_file_data(
						include_bytes!("../assets/icon.ico"),
						None,
					) {
						window::change_icon(id, icon)
					} else {
						Task::none()
					}
				}),
				Task::done(Message::ReadSaves),
			]),
		)
	})
}
