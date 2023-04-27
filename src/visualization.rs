// src/visualization.rs
use crate::categorization::Categorization;
use crate::data::{load_data, Task, TimePeriod};
use chrono::{DateTime, Duration, Utc};
use plotters::prelude::*;
use std::path::PathBuf;

pub fn visualize_data(
    time_period: &TimePeriod,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_time = time_period
        .categorization
        .categories
        .values()
        .flat_map(|tasks| tasks.iter())
        .filter_map(|task| task.time_chunks.first().map(|chunk| chunk.start_time))
        .min()
        .unwrap_or_else(|| Utc::now());

    let max_time = time_period
        .categorization
        .categories
        .values()
        .flat_map(|tasks| tasks.iter())
        .filter_map(|task| {
            task.time_chunks
                .last()
                .and_then(|chunk| chunk.end_time.or(Some(Utc::now())))
        })
        .max()
        .unwrap_or_else(|| Utc::now());

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
        .draw()?;

    let tasks: Vec<&Task> = time_period
        .categorization
        .categories
        .values()
        .flat_map(|tasks| tasks.iter())
        .collect();

    let mut rectangles = vec![];

    for (task_idx, task) in tasks.iter().enumerate() {
        let color = Palette99::pick(task_idx);
        for time_chunk in &task.time_chunks {
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

With these changes, the chart should be drawn and saved correctly to the output path specified. Please try this and let me know if you still face any issues.
