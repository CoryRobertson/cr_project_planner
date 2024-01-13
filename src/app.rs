use chrono::{DateTime, Days, Local, NaiveDate};
use cr_project_planner::project::{Project, ProjectDisplayAction};
use eframe::{Frame, Storage};
use egui::scroll_area::ScrollBarVisibility;
use egui::style::{ScrollStyle, Spacing};
use egui::{Color32, Context, Layout, ScrollArea, Style, ViewportCommand};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use std::thread;
use std::thread::JoinHandle;
use self_update::cargo_crate_version;
use self_update::errors::Error;
use self_update::update::{Release, UpdateStatus};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectPlanner {
    projects: Vec<Project>,
    #[serde(skip)]
    project_name_selected: String,

    #[serde(skip)]
    project_description_selected: String,

    #[serde(skip)]
    start_date_selected: NaiveDate,
    #[serde(skip)]
    end_date_selected: NaiveDate,
    #[serde(skip)]
    editing_project: bool,
    #[serde(skip)]
    editing_project_index: usize,

    #[serde(skip)]
    add_project_open: bool,

    last_open: DateTime<Local>,

    #[serde(skip)]
    first_run: bool,

    #[serde(skip)]
    update_available: Option<Release>,

    auto_update_seen_version: Option<String>,
    
    #[serde(skip)]
    update_thread: Option<JoinHandle<Result<UpdateStatus, Error>>>,

    #[serde(skip)]
    auto_update_status: Option<AutoUpdateStatus>,

    #[serde(skip)]
    showing_about_page: bool,
}

impl ProjectPlanner {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl Default for ProjectPlanner {
    fn default() -> Self {
        Self {
            projects: vec![],
            project_name_selected: "DEFAULT PROJECT NAME".to_string(),
            project_description_selected: "".to_string(),
            start_date_selected: Local::now().date_naive(),
            end_date_selected: Local::now()
                .checked_add_days(Days::new(1))
                .unwrap_or_default()
                .date_naive(),
            editing_project: false,
            editing_project_index: 0,
            add_project_open: false,
            last_open: Local::now(),
            first_run: true,
            update_available: None,
            auto_update_seen_version: None,
            update_thread: None,
            auto_update_status: None,
            showing_about_page: false,
        }
    }
}

fn get_release_list() -> Result<Vec<Release>, Box<dyn std::error::Error>> {
    let list = self_update::backends::github::ReleaseList::configure()
        .repo_owner("CoryRobertson")
        .repo_name("cr_project_planner")
        .build()?
        .fetch()?;

    Ok(list)
}

fn update_program() -> JoinHandle<Result<UpdateStatus, Error>> {
    thread::spawn(|| {
        match self_update::backends::github::UpdateBuilder::new()
            .repo_owner("CoryRobertson")
            .repo_name("cr_project_planner")
            .show_download_progress(true)
            .no_confirm(true)
            .current_version(cargo_crate_version!())
            .build() {
            Ok(updater) => {
                match updater.update_extended() {
                    Ok(status) => {
                        Ok(status)
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            }
            Err(err) => {
                Err(err)
            }
        }
    })
}



impl eframe::App for ProjectPlanner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {

        if self.first_run {
            self.first_run = false;
            if self.last_open.signed_duration_since(Local::now()).num_hours() > 1 {
                match get_release_list() {
                    Ok(list) => {
                        if let Some(release) = list.first() {
                            if let Ok(greater_bump) = self_update::version::bump_is_greater(cargo_crate_version!(),&release.version) {
                                if greater_bump {
                                    self.update_available = Some(release.clone());
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }

        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_style(Style {
                spacing: Spacing {
                    scroll: ScrollStyle::solid(),
                    ..Default::default()
                },
                ..Default::default()
            });

            if !self.add_project_open {
                if ui.button("Create new project").clicked() {
                    self.add_project_open = true;
                }
                ui.separator();
            }

            ScrollArea::vertical()
                .auto_shrink(true)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    self.projects
                        .iter_mut()
                        .enumerate()
                        .for_each(|(project_index, project)| {
                            match project.show(ui) {
                                ProjectDisplayAction::EditClicked => {
                                    self.editing_project = true;
                                    self.editing_project_index = project_index;
                                }
                                ProjectDisplayAction::None => {}
                                ProjectDisplayAction::CloseEditWindow => {}
                            }
                            ui.separator();
                        });
                });

            self.projects
                .retain(|project| project.marked_for_deletion.not());

            ui.with_layout(Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Save & quit").clicked() {
                        if let Some(storage) = _frame.storage_mut() {
                            self.save(storage);
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    }
                    if !self.showing_about_page {
                        if ui.button("About").clicked() {
                            self.showing_about_page = true;
                        }
                    }
                });

            });
        });

        if self.editing_project {
            if let Some(project) = self.projects.get_mut(self.editing_project_index) {
                match project.show_edit_window(ctx) {
                    ProjectDisplayAction::EditClicked => {}
                    ProjectDisplayAction::CloseEditWindow => {
                        self.editing_project = false;
                    }
                    ProjectDisplayAction::None => {}
                }
            } else {
                // stop editing project if it does not exist in the project list
                self.editing_project = false;
            }
        }

        if self.add_project_open {
            egui::Window::new("Add new project").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Project name:");
                    ui.text_edit_singleline(&mut self.project_name_selected);
                });
                ui.horizontal(|ui| {
                    ui.label("Project description:");
                    ui.text_edit_singleline(&mut self.project_description_selected);
                });

                ui.horizontal(|ui| {
                    ui.label("Start date:");
                    ui.push_id(1, |ui| {
                        ui.add(egui_extras::DatePickerButton::new(
                            &mut self.start_date_selected,
                        ));
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("End date:");
                    ui.push_id(2, |ui| {
                        ui.add(egui_extras::DatePickerButton::new(
                            &mut self.end_date_selected,
                        ));
                    });
                });

                match Project::validity_check_new(
                    self.start_date_selected,
                    self.end_date_selected,
                    self.project_name_selected.clone(),
                    &self.projects,
                    self.project_description_selected.clone(),
                ) {
                    Ok(project) => {
                        if ui.button("Add project").clicked() {
                            if !self
                                .projects
                                .iter()
                                .any(|any_project| any_project.project_name == project.project_name)
                            {
                                self.projects.push(project);
                                self.project_name_selected.clear();
                            }
                        }
                    }
                    Err(validity_error) => {
                        let error = validity_error.get_text();
                        ui.colored_label(Color32::LIGHT_RED, error);
                    }
                }
                if ui.button("Close window").clicked() {
                    self.add_project_open = false;
                }
            });
        }

        if let Some(update) = self.update_available.clone() {
            let show_update_screen = match &self.auto_update_seen_version {
                None => {
                    true
                }
                Some(seen_version) => {
                    self_update::version::bump_is_greater(&update.version,&seen_version).unwrap_or(false)
                }
            };

            let update_done = self.auto_update_status.is_some();
            
            if show_update_screen || update_done {
                egui::Window::new("Update Available").show(ctx,|ui| {
                    if !update_done {
                        ui.heading("There is a program update available.");
                        ui.label(format!("Current version: {}", cargo_crate_version!()));
                        ui.label(format!("Updated version version: {}", update.version));
                        let finished = match &self.update_thread {
                            None => {
                                if ui.button("Automagically update").clicked() {
                                    self.update_thread = Some(update_program());
                                }
                                if ui.button("Skip this version").clicked() {
                                    self.auto_update_seen_version = Some(update.version.clone());
                                    self.update_available = None;
                                }
                                false
                            }
                            Some(update_thread) => {
                                if !update_thread.is_finished() {
                                    ui.horizontal(|ui| {
                                        ui.label("Updating...");
                                        ui.spinner();
                                    });
                                    false
                                }
                                else {
                                    true
                                }
                            }
                        };

                        if finished {
                            if let Some(thread) = self.update_thread.take() {
                                self.auto_update_status = Some(thread.join().map(|result| {
                                    match result {
                                        Ok(update) => {
                                            match update {
                                                UpdateStatus::UpToDate => {
                                                    AutoUpdateStatus::UpToDate
                                                }
                                                UpdateStatus::Updated(ver) => {
                                                    AutoUpdateStatus::Updated(ver.version)
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            AutoUpdateStatus::Error
                                        }
                                    }
                                }).unwrap_or(AutoUpdateStatus::Error));
                            }
                        }
                    } else {
                        if let Some(status) = self.auto_update_status.as_ref() {
                            match status {
                                AutoUpdateStatus::Error => {
                                    ui.label("An error occurred while updating");
                                    // TODO: give user more recourse to fix this, or least tell them maybe why it happened ?
                                }
                                AutoUpdateStatus::UpToDate => {
                                    ui.label("You were already on the latest version, whoops!");
                                }
                                AutoUpdateStatus::Updated(_) => {
                                    ui.label("Update finished");
                                    ui.label("Simply close and re-open the program to complete the update.");
                                }
                            }
                            if ui.button("Close this window").clicked() {
                                self.auto_update_status = None;
                            }
                        }

                    }
                });
            }
            
        }

        if self.showing_about_page {
            egui::Window::new("About").show(ctx, |ui| {
                ui.heading("Project Planner");
                ui.label("A project planning and todo list software.");
                ui.separator();
                ui.label("Authors: Cory Robertson");
                ui.label("License: GPL-3.0");
                ui.horizontal(|ui| {
                    ui.label("Github repository:");
                    ui.hyperlink("https://github.com/CoryRobertson/cr_project_planner");
                });
                ui.separator();
                ui.label(format!("Cargo crate version: {}", cargo_crate_version!()));
                ui.separator();
                ui.label(format!("Last open date: {}", self.last_open));
                ui.label(format!(
                    "Auto update seen version: {}",
                    self.auto_update_seen_version.clone().unwrap_or_default()
                ));
                ui.label(format!(
                    "Auto update status: {}",
                    self.auto_update_status.as_ref().map(|status| status.to_text()).unwrap_or("Not checked".to_string())
                ));

                ui.separator();

                if ui.button("Close").clicked() {
                    self.showing_about_page = false;
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}

enum AutoUpdateStatus {
    Error,
    UpToDate,
    Updated(String),
}
impl AutoUpdateStatus {
    pub fn to_text(&self) -> String {
        match self {
            AutoUpdateStatus::Error => {
                "Error".to_string()
            }
            AutoUpdateStatus::UpToDate => {
                "Up to date".to_string()
            }
            AutoUpdateStatus::Updated(version) => {
                version.to_string()
            }
        }
    }
}