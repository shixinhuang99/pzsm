#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod component;
mod consts;
mod pzsm;
mod save;
mod util;

fn main() {
	if let Err(err) = pzsm::run() {
		eprintln!("{}", err);
	}
}
