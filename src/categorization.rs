// src/categorization
use crate::data::Task;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Categorization {
    pub categories: HashMap<String, Vec<Task>>,
}

impl Categorization {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn add_category(&mut self, category: String) {
        if !self.categories.contains_key(&category) {
            self.categories.insert(category.clone(), Vec::new());
        }
        self.categories.entry(category);
    }

    pub fn add_task_to_category(&mut self, task: Task) {
        let (category, task_description) = Self::extract_category_from_description(&task.name);
        let mut task = task.clone();
        task.name = task_description;
        let category = &category;

        if let Some(tasks) = self.categories.get_mut(&category.clone()) {
            tasks.push(task);
        } else {
            // If category doesn't exist, create it and add the task
            self.add_category(category.clone());
            self.add_task_to_category(task);
        }
    }

    pub fn extract_category_from_description(description: &str) -> (String, String) {
        let mut category = String::from("Uncategorized");
        let mut task_description = description.to_string();
        //println!("Description(Before): {}", description.to_string());

        if let Some(hashtag_index) = description.find('#') {
            let category_start = hashtag_index;
            let (before, after) = description.split_at(category_start);
            let mut after_iter = after.split_whitespace();
            if let Some(first_word) = after_iter.next() {
                category = first_word.to_string();
                //println!("Extracted category: {}", first_word.to_string());
                task_description = format!(
                    "{}{}",
                    before.trim(),
                    after_iter.collect::<Vec<_>>().join(" ")
                );
                //println!("Task Description: {}", task_description);
            }
        }

        (category, task_description)
    }
}
