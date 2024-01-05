use chrono::NaiveDate;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl Project {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate, project_name: String) -> Self {
        Self {
            project_name,
            start_date,
            end_date,
        }
    }

    pub fn show(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(&self.project_name);
            ui.label(format!("{}", self.start_date));
            ui.label(format!("{}", self.end_date));
        });
    }

    pub fn validity_check_new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        project_name: String,
    ) -> Option<Self> {
        if start_date >= end_date {
            return None;
        }
        if project_name.trim().is_empty() {
            return None;
        }

        Some(Self::new(start_date, end_date, project_name))
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
