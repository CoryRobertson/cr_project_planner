#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::ProjectPlanner;
use eframe::NativeOptions;

mod app;

const APP_NAME: &str = "cr_project_planner";

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        persist_window: true,
        // TODO: make window open with default size that shows all ui elements properly
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Box::new(ProjectPlanner::new(cc))),
    )
}
