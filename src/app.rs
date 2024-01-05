use chrono::{Days, NaiveDate};
use cr_project_planner::project::Project;
use eframe::{Frame, Storage};
use egui::{Color32, Context};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectPlanner {
    projects: Vec<Project>,
    #[serde(skip)]
    project_name_selected: String,
    #[serde(skip)]
    start_date_selected: NaiveDate,
    #[serde(skip)]
    end_date_selected: NaiveDate,
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
            start_date_selected: chrono::offset::Local::now().date_naive(),
            end_date_selected: chrono::offset::Local::now()
                .checked_add_days(Days::new(1))
                .unwrap_or_default()
                .date_naive(),
        }
    }
}

impl eframe::App for ProjectPlanner {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {

        // TODO: implement a floating panel that lets you edit a task when an edit task button is clicked

        egui::CentralPanel::default().show(ctx, |ui| {
            self.projects.iter_mut().for_each(|project| {
                project.show(ui);
            });

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Project name:");
                    ui.text_edit_singleline(&mut self.project_name_selected);
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
                ) {
                    Ok(project) => {
                        if ui.button("Add project").clicked() {
                            self.projects.push(project);
                        }
                    }
                    Err(validity_error) => {
                        let error = validity_error.get_text();
                        ui.colored_label(
                            Color32::LIGHT_RED,
                            error,
                        );
                    }
                }
            });

            if ui.button("clear projects").clicked() {
                self.projects.clear();
            }
            if ui.button("remove last project").clicked() {
                self.projects.pop();
            }
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
