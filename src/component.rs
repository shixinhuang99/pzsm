use iced::{
	widget::{
		button, column, container, horizontal_rule, row, rule, text,
		vertical_rule, Button, Container, Row, Rule,
	},
	Element, Length, Padding, Theme,
};

use crate::{backup::Backup, save::Save};

fn divider_style(theme: &Theme) -> rule::Style {
	let palette = theme.extended_palette();

	rule::Style {
		color: palette.background.weak.color,
		width: 1,
		radius: 0.into(),
		fill_mode: rule::FillMode::Full,
	}
}

pub fn horizontal_divider() -> Rule<'static> {
	horizontal_rule(1).style(divider_style)
}

pub fn vertical_divider() -> Rule<'static> {
	vertical_rule(1).style(divider_style)
}

fn item_btn_padding() -> Padding {
	Padding::new(12.0)
}

pub fn save_button<'a, T: 'a>(
	save: &'a Save,
	msg: impl FnOnce(Save) -> T,
	is_primary: impl FnOnce(&str) -> bool,
) -> Button<'a, T> {
	let mut btn = button(column([
		text(&save.name).into(),
		text(&save.update_time).size(12).into(),
	]))
	.width(Length::Fill)
	.padding(item_btn_padding())
	.on_press(msg(save.clone()));

	if is_primary(&save.name) {
		btn = btn.style(button::primary);
	} else {
		btn = btn.style(button::secondary);
	}

	btn
}

pub fn backup_button<'a, T: 'a>(
	backup: &'a Backup,
	msg: impl FnOnce(Backup) -> T,
	is_primary: impl FnOnce(&str) -> bool,
) -> Button<'a, T> {
	let mut btn = button(text(&backup.name))
		.width(Length::Fill)
		.padding(item_btn_padding())
		.on_press(msg(backup.clone()));

	if is_primary(&backup.name) {
		btn = btn.style(button::primary);
	} else {
		btn = btn.style(button::secondary);
	}

	btn
}

pub fn panel<'a, T>(content: impl Into<Element<'a, T>>) -> Container<'a, T> {
	container(content.into())
		.width(Length::FillPortion(2))
		.padding(Padding::default().top(4).right(4).bottom(4).left(6))
}

pub fn panel_centered_text<T>(s: &str) -> Container<'_, T> {
	container(text(s))
		.center_x(Length::FillPortion(2))
		.center_y(Length::Fill)
}

pub fn body_centered_text<T>(s: &str) -> Container<'_, T> {
	container(text(s))
		.center_x(Length::Fill)
		.center_y(Length::Fill)
}

pub fn row_between<'a, T: 'a, L, R>(
	height: u16,
	left: L,
	right: R,
) -> Row<'a, T>
where
	L: Into<Element<'a, T>>,
	R: Into<Element<'a, T>>,
{
	let left_c = container(left.into())
		.center_y(height)
		.align_left(Length::Fill)
		.padding(Padding::default().left(12));

	let right_c = container(right.into())
		.center_y(height)
		.align_right(Length::Shrink)
		.padding(Padding::default().right(12));

	row([left_c.into(), right_c.into()])
}
