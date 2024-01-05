use chrono::NaiveDate;
use egui::Ui;
use serde::{Deserialize, Serialize};

mod task;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    // TODO: add a task vector for each project
}

impl Project {
    fn new(start_date: NaiveDate, end_date: NaiveDate, project_name: String) -> Self {
        Self {
            project_name,
            start_date,
            end_date,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(&self.project_name);
            ui.label(format!("{}", self.start_date));
            ui.label(format!("{}", self.end_date));

            // TODO: add checkbox steps that a project can have

        });
    }

    pub fn validity_check_new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        project_name: String,
    ) -> Result<Self, ProjectValidityError> {
        if start_date >= end_date {
            return Err(ProjectValidityError::DateOrdinalityError);
        }
        if project_name.trim().is_empty() {
            return Err(ProjectValidityError::ProjectNameError);
        }

        Ok(Self::new(start_date, end_date, project_name))
    }
}

pub enum ProjectValidityError {
    ProjectNameError,
    DateOrdinalityError,
}

impl ProjectValidityError {
    pub fn get_text(&self) -> String {
        match self {
            ProjectValidityError::ProjectNameError => {
                "Project name needs to have text".to_string()
            }
            ProjectValidityError::DateOrdinalityError => {
                "Project start date needs to proceed project end date".to_string()
            }
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project_name: "Unnamed Project".to_string(),
            start_date: Default::default(),
            end_date: Default::default(),
        }
    }
}
