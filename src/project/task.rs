use chrono::{DateTime, Datelike, Local, Timelike};
use egui::{RichText, Ui};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Task {
    pub text: String,

    pub description: String,

    completed: bool,
    completed_date: Option<DateTime<Local>>,
    #[serde(skip)]
    pub marked_for_deletion: bool,
}

impl Task {
    pub fn new(text: String, description: String) -> Self {
        Self {
            text,
            description,
            completed: false,
            completed_date: None,
            marked_for_deletion: false,
        }
    }

    pub fn get_completed(&self) -> bool {
        self.completed
    }
    pub fn set_completed(&mut self, completed: bool) {
        self.completed = completed;
        if completed {
            self.completed_date = Some(Local::now());
        } else {
            self.completed_date = None;
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let mut completed = self.get_completed();
        ui.horizontal(|ui| {
            if self.completed {
                ui.checkbox(&mut completed, RichText::new(&self.text).strikethrough());
            } else {
                ui.checkbox(&mut completed, RichText::new(&self.text));
            }

            if let Some(complete_date) = self.get_complete_date() {
                ui.label(format!(
                    "Completed on: {}-{}-{} {}:{} {}",
                    complete_date.year(),
                    complete_date.month(),
                    complete_date.day(),
                    complete_date.hour12().1,
                    complete_date.minute(),
                    if complete_date.hour12().0 { "PM" } else { "AM" }
                ));
            }
        });
        if !self.description.is_empty() {
            ui.horizontal(|ui| {
                ui.separator();
                if self.completed {
                    ui.label(RichText::new(self.description.to_string()).strikethrough());
                } else {
                    ui.label(&self.description);
                }
            });
        }

        if self.get_completed() != completed {
            self.set_completed(completed);
        }
    }

    pub fn get_complete_date(&self) -> Option<&DateTime<Local>> {
        self.completed_date.as_ref()
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            text: "DEFAULT TASK TEXT".to_string(),
            description: "".to_string(),
            completed: false,
            completed_date: None,
            marked_for_deletion: false,
        }
    }
}

impl PartialEq<Self> for Task {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.description == other.description
    }
}
