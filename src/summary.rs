// src/summary.rs
use crate::categorization::Categorization;
use crate::data::{Task, TaskStatus};
use chrono::{Duration, Utc};
use std::collections::HashMap;

pub fn print_summary(time_period: &HashMap<String, Vec<Task>>) {
    println!("Task Summary:");
    print_summary_with_duration(time_period, Duration::max_value(), None);
}

pub fn print_summary_with_duration(
    time_period: &HashMap<String, Vec<Task>>,
    duration: Duration,
    category_name: Option<String>,
) {
    let now = Utc::now();

    for (category, tasks) in time_period {
        if let Some(ref name) = category_name {
            if name.to_string() != category.to_string() {
                continue;
            }
        }

        println!("Category: {:?}", category);

        let filtered_tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                if let Some(chunk) = task.time_chunks.last() {
                    now.signed_duration_since(chunk.start_time) <= duration
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let (total_duration, running_task_count, paused_task_count, stopped_task_count) =
            calculate_summary(&filtered_tasks);

        println!("Total duration: {}", format_duration(total_duration));
        println!("Running tasks: {}", running_task_count);
        println!("Paused tasks: {}", paused_task_count);
        println!("Stopped tasks: {}", stopped_task_count);
        println!("");
    }
}

fn calculate_summary(tasks: &[Task]) -> (Duration, usize, usize, usize) {
    let mut total_duration = Duration::zero();
    let mut running_task_count = 0;
    let mut paused_task_count = 0;
    let mut stopped_task_count = 0;

    for task in tasks {
        match task.status {
            TaskStatus::Running => {
                running_task_count += 1;
                total_duration = total_duration
                    + task
                        .time_chunks
                        .iter()
                        .filter_map(|chunk| chunk.end_time.map(|t| t - chunk.start_time))
                        .fold(Duration::zero(), |acc, x| acc + x)
                    + (Utc::now() - task.time_chunks.last().unwrap().start_time);
            }
            TaskStatus::Paused => {
                paused_task_count += 1;
                total_duration = total_duration
                    + task
                        .time_chunks
                        .iter()
                        .filter_map(|chunk| chunk.end_time.map(|t| t - chunk.start_time))
                        .fold(Duration::zero(), |acc, x| acc + x);
            }
            TaskStatus::Stopped => {
                stopped_task_count += 1;
                total_duration = total_duration
                    + task
                        .time_chunks
                        .iter()
                        .filter_map(|chunk| chunk.end_time.map(|t| t - chunk.start_time))
                        .fold(Duration::zero(), |acc, x| acc + x);
            }
        }
    }

    (
        total_duration,
        running_task_count,
        paused_task_count,
        stopped_task_count,
    )
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
}
