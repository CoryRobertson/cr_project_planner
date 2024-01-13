use crate::project::task::Task;
use chrono::{DateTime, Local, NaiveDate};
use egui::scroll_area::ScrollBarVisibility;
use egui::{ScrollArea, Ui};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod task;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(skip)]
    pub marked_for_deletion: bool,

    pub tasks: Vec<Task>,

    #[serde(skip)]
    next_task_text: String,

    #[serde(skip)]
    next_task_description: String,

    creation_date: DateTime<Local>,

    #[serde(skip)]
    task_editing: bool,

    #[serde(skip)]
    selected_editing_task: usize,

    pub description: String,

    uuid: Uuid,
}

pub enum ProjectDisplayAction {
    EditClicked,
    CloseEditWindow,
    None,
}

impl Project {
    fn new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        project_name: String,
        description: String,
    ) -> Self {
        Self {
            project_name,
            start_date,
            end_date,
            marked_for_deletion: false,
            tasks: vec![],
            next_task_text: "".to_string(),
            next_task_description: "".to_string(),
            creation_date: Local::now(),
            task_editing: false,
            selected_editing_task: 0,
            description,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> ProjectDisplayAction {
        let mut action = ProjectDisplayAction::None;
        ui.horizontal(|ui| {
            ui.push_id(self.uuid, |ui| {
                ScrollArea::horizontal()
                    .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                    .show(ui, |ui| {
                        ui.heading(&self.project_name);
                        ui.separator();
                        if !self.description.is_empty() {
                            ui.label(&self.description);
                            ui.separator();
                        }
                        ui.label(format!("Start date: {}", self.start_date));
                        ui.separator();
                        ui.label(format!("End date: {}", self.end_date));
                        ui.separator();
                        let time_left= self.end_date.signed_duration_since(Local::now().date_naive());
                        ui.label(format!("Days until due: {}", time_left.num_days()));
                        ui.separator();
                        if ui.button("Edit").clicked() {
                            action = ProjectDisplayAction::EditClicked
                        }
                        ui.separator();
                    });
            });
        });
        ui.separator();

        self.tasks.retain(|task| !task.marked_for_deletion);

        ui.push_id(self.uuid, |ui| {
            ui.collapsing("Tasks", |ui| {
                self.tasks.iter_mut().for_each(|task| {
                    task.show(ui);
                });
            });
        });

        action
    }

    pub fn show_edit_window(&mut self, ctx: &egui::Context) -> ProjectDisplayAction {
        let mut action = ProjectDisplayAction::None;
        egui::Window::new("Project Editor").show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.project_name)
                .on_hover_text("Project name");
            ui.text_edit_multiline(&mut self.description)
                .on_hover_text("Project description");
            let mut start_date = self.start_date.clone();
            let mut end_date = self.end_date.clone();
            ui.horizontal(|ui| {
                ui.label("Start date:");
                ui.push_id(3, |ui| {
                    ui.add(egui_extras::DatePickerButton::new(&mut start_date));
                });
            });
            ui.horizontal(|ui| {
                ui.label("End date:");
                ui.push_id(4, |ui| {
                    ui.add(egui_extras::DatePickerButton::new(&mut end_date));
                });
            });

            if start_date < end_date {
                self.start_date = start_date;
                self.end_date = end_date;
            }

            ui.label("Task name: ");
            ui.text_edit_singleline(&mut self.next_task_text);
            ui.label("Task description: ");
            ui.text_edit_singleline(&mut self.next_task_description);
            if ui.button("Add task").clicked() {
                self.tasks.push(Task::new(
                    self.next_task_text.clone(),
                    self.next_task_description.clone(),
                ));
                self.next_task_text.clear();
            }

            ui.collapsing("Task editor", |ui| {
                ScrollArea::vertical()
                    .auto_shrink(true)
                    .max_height(150.0)
                    .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                    .show(ui, |ui| {
                        let (mut re_aranged, mut re_aranged_task_index, mut re_aranged_new_index) =
                            (false, 0, 0);
                        let task_length = self.tasks.len();
                        self.tasks
                            .iter_mut()
                            .enumerate()
                            .for_each(|(task_index, task)| {
                                let mut new_index = task_index;
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::widgets::DragValue::new(&mut new_index)
                                            .clamp_range(0..=(task_length - 1)),
                                    );
                                    ui.label(format!("{}", task.text));
                                    if ui.button("Edit").clicked() {
                                        self.task_editing = true;
                                        self.selected_editing_task = task_index;
                                    }
                                    if ui.button("Delete task").clicked() {
                                        task.marked_for_deletion = true;
                                    }
                                });
                                if new_index != task_index {
                                    re_aranged = true;
                                    re_aranged_task_index = task_index;
                                    re_aranged_new_index = new_index;
                                }
                            });
                        if re_aranged {
                            if let Some(task_clone) = self.tasks.get(re_aranged_task_index).cloned()
                            {
                                self.tasks.remove(re_aranged_task_index);
                                self.tasks.insert(re_aranged_new_index, task_clone.clone());
                            }
                        }
                    });
            });

            if ui.button("Remove finished tasks").clicked() {
                self.tasks.retain(|task| !task.get_completed());
            }

            if ui
                .button("Delete project")
                .on_hover_text("Double click this button to delete the project")
                .double_clicked()
            {
                self.marked_for_deletion = true;
            }

            if ui.button("Close editing window").clicked() {
                action = ProjectDisplayAction::CloseEditWindow;
            }
        });

        if self.task_editing {
            if let Some(task_to_edit) = self.tasks.get_mut(self.selected_editing_task) {
                egui::Window::new("Task Editor").show(ctx, |ui| {
                    ui.text_edit_singleline(&mut task_to_edit.text);
                    ui.text_edit_singleline(&mut task_to_edit.description);

                    if ui.button("Close task editor").clicked() {
                        self.task_editing = false;
                    }
                });
            } else {
                self.task_editing = false;
            }
        }

        action
    }

    pub fn validity_check_new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        project_name: String,
        projects: &[Project],
        description: String,
    ) -> Result<Self, ProjectValidityError> {
        if start_date >= end_date {
            return Err(ProjectValidityError::DateOrdinalityError);
        }
        if project_name.trim().is_empty() {
            return Err(ProjectValidityError::ProjectNameError);
        }

        if projects
            .iter()
            .any(|project| project.project_name == project_name)
        {
            return Err(ProjectValidityError::ProjectAlreadyExists);
        }

        Ok(Self::new(start_date, end_date, project_name, description))
    }
}

pub enum ProjectValidityError {
    ProjectNameError,
    DateOrdinalityError,
    ProjectAlreadyExists,
}

impl ProjectValidityError {
    pub fn get_text(&self) -> String {
        match self {
            ProjectValidityError::ProjectNameError => "Project name needs to have text".to_string(),
            ProjectValidityError::DateOrdinalityError => {
                "Project start date needs to proceed project end date".to_string()
            }
            ProjectValidityError::ProjectAlreadyExists => "Project name already exists".to_string(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project_name: "Unnamed Project".to_string(),
            start_date: Default::default(),
            end_date: Default::default(),
            marked_for_deletion: false,
            tasks: vec![],
            next_task_text: "".to_string(),
            next_task_description: "".to_string(),
            creation_date: Local::now(),
            task_editing: false,
            selected_editing_task: 0,
            description: "".to_string(),
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq<Self> for Project {
    fn eq(&self, other: &Self) -> bool {
        self.project_name == other.project_name
            && self.description == other.description
            && self.tasks == other.tasks
            && self.start_date == other.start_date
            && self.end_date == other.end_date
    }
}
