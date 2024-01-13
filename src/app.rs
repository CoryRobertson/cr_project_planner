use chrono::{Days, NaiveDate};
use cr_project_planner::project::{Project, ProjectDisplayAction};
use eframe::{Frame, Storage};
use egui::scroll_area::ScrollBarVisibility;
use egui::style::{ScrollStyle, Spacing};
use egui::{Color32, Context, Layout, ScrollArea, Style, ViewportCommand};
use serde::{Deserialize, Serialize};
use std::ops::Not;

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
            start_date_selected: chrono::offset::Local::now().date_naive(),
            end_date_selected: chrono::offset::Local::now()
                .checked_add_days(Days::new(1))
                .unwrap_or_default()
                .date_naive(),
            editing_project: false,
            editing_project_index: 0,
            add_project_open: false,
        }
    }
}

impl eframe::App for ProjectPlanner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
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
                if ui.button("Save & quit").clicked() {
                    if let Some(storage) = _frame.storage_mut() {
                        self.save(storage);
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
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
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
