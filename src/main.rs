mod categorization;
mod config;
mod data;
mod summary;
mod visualization;

use crate::categorization::Categorization;
use crate::config::AppConfig;
use crate::data::{load_data, save_data, Task, TaskStatus, TimeChunk, TimePeriod};
use crate::summary::print_summary_with_duration;
use crate::visualization::visualize_data;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "time_tracker")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Start {
        #[structopt(help = "Task name or description")]
        task: Vec<String>,
    },
    Stop,
    Pause,
    Resume,
    Status,
    List,
    Export {
        #[structopt(long, parse(from_os_str), help = "File path to export data")]
        file_path: PathBuf,
    },
    Summary {
        #[structopt(help = "Time period for the summary: daily, weekly, or monthly")]
        period: String,
    },
    Configure {
        #[structopt(long, help = "Custom storage location for data file")]
        storage_location: String,
    },
    Visualize,
    Clear,
}

fn main() {
    let opt = Opt::from_args();

    let app_config = AppConfig::load();
    let storage_location = app_config.storage_location.unwrap_or_else(|| {
        let mut default_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        default_path.push("time_tracker");
        default_path.push("tasks.json");
        default_path
    });

    // Change the type of `time_period`
    let mut time_period: TimePeriod = load_data(&storage_location);

    // Update function calls accordingly
    match opt.command {
        Command::Start { task } => {
            // Join the task Vec<String> with spaces
            println!("{:?}", task);
            let task = task.join(" ");
            start_task(&mut time_period, &task, Utc::now(), &storage_location)
        }
        Command::Stop => stop_task(&mut time_period, &storage_location),
        Command::Pause => pause_task(&mut time_period),
        Command::Resume => resume_task(&mut time_period),
        Command::List => list_tasks(&mut time_period),
        Command::Status => clock(&mut time_period),
        Command::Export { file_path } => export_data(file_path),
        Command::Summary { period } => generate_summary(&time_period, period, None),
        Command::Configure { storage_location } => {
            configure_app(Some(PathBuf::from(storage_location + "tasks.json")))
        }
        Command::Visualize => visualize(&time_period),
        Command::Clear => clear(&mut time_period, &storage_location),
    }
}

fn start_task(
    time_period: &mut TimePeriod,
    name: &str,
    start_time: DateTime<Utc>,
    storage_location: &PathBuf,
) {
    let category = categorization::Categorization::extract_category_from_description(name);
    //println!("Category: {:?}", category);
    if time_period
        .categorization
        .categories
        .contains_key(&category.0.clone())
    {
    } else {
        time_period.categorization.add_category(category.0.clone());
    }

    let existing_task = time_period
        .categorization
        .categories
        .get_key_value(&category.0)
        .unwrap()
        .1
        .iter()
        .find(|task| task.name == name);

    match existing_task {
        Some(task) => {
            println!("Found existing task");
            let mut task = task.clone();
            match task.status {
                TaskStatus::Running => {
                    println!("Task already running, no changes made.");
                }
                TaskStatus::Paused => {
                    println!("Task already exists, resuming...");

                    task.status = TaskStatus::Running
                }
                TaskStatus::Stopped => {
                    task.status = TaskStatus::Running;
                    task.time_chunks.push(TimeChunk {
                        start_time,
                        end_time: None,
                    });
                }
            }
        }
        None => {
            println!("Creating new task");
            let new_task = Task {
                name: name.to_string(),
                time_chunks: vec![TimeChunk {
                    start_time,
                    end_time: None,
                }],
                paused_duration: Duration::from_secs(0),
                status: TaskStatus::Running,
            };
            time_period.categorization.add_task_to_category(new_task)
        }
    }
    save_data(storage_location, &time_period).unwrap();
}

fn stop_task(time_period: &mut TimePeriod, storage_location: &PathBuf) {
    let current_task = time_period
        .categorization
        .categories
        .values_mut()
        .flat_map(|tasks| tasks.iter_mut())
        .find(|task| task.status == TaskStatus::Running);

    if let Some(task) = current_task {
        task.status = TaskStatus::Stopped;
        task.time_chunks.last_mut().unwrap().end_time = Some(Utc::now());
        println!("Stopped current task: {:?}", task.name);
    } else {
        println!("No task is currently running.");
    }
    save_data(storage_location, &time_period).unwrap();
}

fn pause_task(time_period: &mut TimePeriod) {
    let current_task = time_period
        .categorization
        .categories
        .values_mut()
        .flat_map(|tasks| tasks.iter_mut())
        .find(|task| task.status == TaskStatus::Running);

    if let Some(task) = current_task {
        task.status = TaskStatus::Paused;
        task.time_chunks.last_mut().unwrap().end_time = Some(Utc::now());
        println!("Paused current task: {:?}", task.name);
    } else {
        println!("No task is currently running.");
    }
}

fn resume_task(time_period: &mut TimePeriod) {
    let paused_task = time_period
        .categorization
        .categories
        .values_mut()
        .flat_map(|tasks| tasks.iter_mut())
        .find(|task| task.status == TaskStatus::Paused);

    if let Some(task) = paused_task {
        task.status = TaskStatus::Running;
        task.time_chunks.push(TimeChunk {
            start_time: Utc::now(),
            end_time: None,
        });
        println!("Resumed task: {:?}", task.name);
    } else {
        println!("No paused task found.");
    }
}

fn clock(time_period: &mut TimePeriod) {
    for tasks in time_period.categorization.categories.values_mut() {
        for task in tasks {
            if let data::TaskStatus::Running = task.status {
                task.time_spent();
                break;
            }
        }
    }
}

fn list_tasks(time_period: &TimePeriod) {
    println!("Listing tasks");
    for (category, tasks) in &time_period.categorization.categories {
        println!("Category: {:?}", category);
        for task in tasks {
            println!(
                "{} - {} - {}",
                task.name,
                task.time_chunks[0].start_time.to_rfc3339(),
                task.time_chunks
                    .last()
                    .and_then(|chunk| chunk.end_time.map(|t| t.to_rfc3339()))
                    .unwrap_or_else(|| String::from("N/A")),
            );
            task.time_spent();
        }
    }
}

fn export_data(file_path: PathBuf) {
    println!("Exporting data to {:?}", file_path);
    // Implement functionality in the appropriate module
}

fn generate_summary(time_period: &TimePeriod, period: String, category: Option<String>) {
    let time_period = &time_period.categorization.categories;
    match period.to_lowercase().as_str() {
        "day" => {
            // Summary for the last day
            println!("Time spent in the last day: ");
            print_summary_with_duration(time_period, chrono::Duration::days(1), category);
        }
        "week" => {
            // Summary for the last week
            println!("Time spent in the last week: ");
            print_summary_with_duration(time_period, chrono::Duration::weeks(1), category);
        }
        "month" => {
            // Summary for the last month
            println!("Time spent in the last month: ");
            print_summary_with_duration(time_period, chrono::Duration::days(30), category);
        }
        x => {
            let period = x.parse().unwrap_or(1);
            print_summary_with_duration(time_period, chrono::Duration::days(period), category)
        }
    }
}

fn configure_app(storage_location: Option<PathBuf>) {
    println!(
        "Configuring app with storage location: {:?}",
        storage_location
    );
    let mut app_config = AppConfig::load();
    app_config.storage_location = storage_location;
    app_config.save().unwrap();
}

fn visualize(time_period: &TimePeriod) {
    println!("Visualizing time tracking data");
    visualize_data(
        time_period,
        &PathBuf::from("/home/lowband/dev/rust/time_tracker/chart.png"),
    )
    .unwrap();
}

fn clear(time_period: &mut TimePeriod, storage_location: &PathBuf) {
    time_period.categorization = Categorization::new();
    save_data(storage_location, &time_period).unwrap();
    println!("Cleared all data");
}
