// src/visualization.rs
use crate::categorization::Categorization;
use crate::data::{load_data, Task, TimePeriod};
use chrono::TimeZone;
use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;
use plotters::prelude::*;
use std::path::PathBuf;

pub fn visualize_data(
    time_period: &TimePeriod,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Output path: {:?}", output_path); // Debug statement
    let root = BitMapBackend::new(output_path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let now = Tz::US__Mountain.from_utc_datetime(&Utc::now().naive_utc());
    let today_4am = now.date_naive().and_hms_opt(4, 0, 0).unwrap();
    let min_time = if now.time().hour() >= 4 {
        today_4am
    } else {
        today_4am - Duration::days(1)
    };
    let max_time = min_time + Duration::days(1);

    let duration = max_time - min_time;
    let duration_secs = duration.num_seconds() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption("Time Tracker Visualization", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(
            min_time.timestamp()..max_time.timestamp(),
            0..time_period.categorization.categories.len(),
        )?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .y_desc("Tasks")
        .axis_desc_style(("sans-serif", 15))
        .x_labels(10) // Controls the number of labels on the x-axis
        .x_label_formatter(&|timestamp| {
            let datetime = Tz::US__Mountain
                .from_utc_datetime(&NaiveDateTime::from_timestamp_opt(*timestamp, 0).unwrap());
            datetime.format("%H:%M:%S").to_string()
        })
        .draw()?;

    let tasks: Vec<&Task> = time_period
        .categorization
        .categories
        .values()
        .flat_map(|tasks| tasks.iter())
        .collect();

    println!("Number of tasks: {}", tasks.len()); // Debug statement

    let mut rectangles = vec![];

    for (task_idx, task) in tasks.iter().enumerate() {
        for time_chunk in &task.time_chunks {
            let color = Palette99::pick(task_idx);
            let start = time_chunk.start_time.timestamp();
            let end = time_chunk
                .end_time
                .map(|t| t.timestamp())
                .unwrap_or_else(|| Utc::now().timestamp());

            rectangles.push(Rectangle::new(
                [(start, task_idx), (end, task_idx + 1)],
                color,
            ));
        }
    }

    chart.draw_series(rectangles)?;

    Ok(())
}
