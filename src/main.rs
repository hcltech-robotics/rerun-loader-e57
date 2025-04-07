use std::collections::HashSet;
use std::env;
use anyhow::{Context, Result};
use e57::{CartesianCoordinate, E57Reader};
use rerun::{Points3D, EXTERNAL_DATA_LOADER_INCOMPATIBLE_EXIT_CODE};
use rerun::{RecordingStreamBuilder, Vec3D};

/// Command line arguments for the E57 Rerun data loader.
#[derive(argh::FromArgs, Debug)]
#[argh(description = "Load E57 point clouds and stream them to Rerun")]
struct Args {
    #[argh(positional)]
    filepath: std::path::PathBuf,

    #[argh(option, description = "optional recommended ID for the application")]
    application_id: Option<String>,

    #[argh(option, description = "optional recommended ID for the application for existing applications")]
    opened_application_id: Option<String>,

    #[argh(option, description = "optional recommended ID for the recording")]
    recording_id: Option<String>,

    #[argh(option, description = "optional recommended ID for the recording for existing applications")]
    opened_recording_id: Option<String>,

    #[argh(option, description = "optional prefix for all entity paths")]
    entity_path_prefix: Option<String>,

    #[argh(
        arg_name = "static",
        switch,
        description = "optionally mark data to be logged statically"
    )]
    #[allow(dead_code)]
    static_: bool,

    #[argh(
        option,
        description = "optional timestamps to log at (e.g. --time sim_time=1709203426)"
    )]
    #[allow(dead_code)]
    time: Vec<String>,

    #[argh(
        option,
        description = "optional sequences to log at (e.g. --sequence sim_frame=42)"
    )]
    #[allow(dead_code)]
    sequence: Vec<String>,
}

fn extension(path: &std::path::Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .to_string_lossy()
        .to_string()
}

fn get_allowed_scans() -> Option<HashSet<usize>> {
    let allowed_scans: Option<HashSet<usize>> = match env::var("RERUN_E57_DISPLAY_SCANS") {
        Ok(val) => {
            let set = val
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect::<HashSet<_>>();
            Some(set)
        }
        Err(_) => None,
    };
    
    allowed_scans
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let is_file = args.filepath.is_file();
    let is_e57 = extension(&args.filepath) == "e57";

    if !is_file || !is_e57 {
        #[allow(clippy::exit)]
        std::process::exit(EXTERNAL_DATA_LOADER_INCOMPATIBLE_EXIT_CODE);
    }

    let mut reader = E57Reader::from_file(&args.filepath)
        .with_context(|| format!("Failed to read E57 file: {:?}", args.filepath))?;

    let rec = {
        
        let app_id = args
            .opened_application_id
            .as_deref()
            .unwrap_or(args.application_id.as_deref().unwrap_or("rerun_e57_loader"));

        let mut rec = RecordingStreamBuilder::new(
            app_id,
        );

        if let Some(recording_id) = &args.recording_id {
            rec = rec.recording_id(recording_id);
        } else if let Some(opened_recording_id) = &args.opened_recording_id {
            rec = rec.recording_id(opened_recording_id);
            
        }

        rec.stdout()?
    };

    // if let Some(timepoint) = timepoint_from_args(&args) {
    //     rec.set_timepoint(timepoint);
    // }

    let allowed_scans = get_allowed_scans();

    let entity_path_prefix = args
        .entity_path_prefix
        .as_deref()
        .unwrap_or("e57_pointcloud");

    let pointclouds = reader.pointclouds();
    for (index, pointcloud) in pointclouds.iter().enumerate() {
        if !pointcloud.has_cartesian() {
            println!("Point cloud #{index} has no XYZ data, skipping...");
            continue;
        }

        if pointcloud.records < 1 {
            println!("Point cloud #{index} is empty, skipping...");
            continue;
        }

        if let Some(allowed_scans) = &allowed_scans {
            if !allowed_scans.contains(&index) {
                continue;
            }
        }

        let iter = reader
            .pointcloud_simple(pointcloud)
            .context("Unable to get simple point cloud iterator")?;

        let mut chunk_idx = 0;
        let chunk_size = 1000000;

        let mut buffer = Vec::with_capacity(chunk_size);
        let mut color_buffer = Vec::with_capacity(chunk_size);

        // if let Some(transform) = &pointcloud.transform {
        //     let translation = &transform.translation;
        //     let rotation = &transform.rotation;

        //     let translation = Vec3D::new(
        //         translation.x as f32,
        //         translation.y as f32,
        //         translation.z as f32,
        //     );
        //     let rotation = rerun::Rotation3D::Quaternion(RotationQuat(Quaternion([
        //         rotation.x as f32,
        //         rotation.y as f32,
        //         rotation.z as f32,
        //         rotation.w as f32,
        //     ])));

        //     let entity_path = format!("{entity_path_prefix}/scan_{index}");
        //     rec.log_static(
        //         entity_path,
        //         &rerun::Transform3D::from_translation_rotation(translation, rotation),
        //     )?;
        // }

        rec.set_time_seconds("default", 0);
        if let Some(transform) = &pointcloud.transform {
            let translation = &transform.translation;

            let translation = [(
                translation.x as f32,
                translation.y as f32,
                translation.z as f32,
            )];
            rec.log(
                format!("{entity_path_prefix}/scan_{index}/point"),
                &Points3D::new(translation)
                    .with_colors([rerun::Color::from_rgb(255, 0, 0)])
                    .with_radii([0.15_f32])
                    .with_labels([format!("Scan {index}")])
            )?;
        }

        for point_result in iter {
            let p = match point_result {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Skipping point due to error: {e}");
                    continue;
                }
            };

            if let CartesianCoordinate::Valid { x, y, z } = p.cartesian {
                buffer.push(Vec3D::new(x as f32, y as f32, z as f32));
                let color = match p.color {
                    Some(color) => rerun::Color::from_rgb(
                        (color.red * 255.0) as u8,
                        (color.green * 255.0) as u8,
                        (color.blue * 255.0) as u8,
                    ),
                    _ => rerun::Color::from_rgb(255, 255, 255),
                };
                color_buffer.push(color)
            }

            if buffer.len() >= chunk_size {
                rec.log(
                    format!("{entity_path_prefix}/scan_{index}/chunk_{chunk_idx}"),
                    &rerun::Points3D::new(std::mem::take(&mut buffer))
                        .with_colors(color_buffer.clone()),
                )?;

                buffer.clear();
                color_buffer.clear();
                chunk_idx += 1;
            }
        }

        if !buffer.is_empty() {
            rec.log(
                format!("{entity_path_prefix}/scan_{index}/chunk_{chunk_idx}"),
                &rerun::Points3D::new(buffer).with_colors(color_buffer.clone()),
            )?;
        }
    }

    Ok(())
}

// fn timepoint_from_args(args: &Args) -> Option<rerun::TimePoint> {
//     if args.time.is_empty() && args.sequence.is_empty() {
//         return None;
//     }

//     let mut timepoint = rerun::TimePoint::default();

//     for time_str in &args.time {
//         if let Some((timeline_name, time)) = time_str.split_once('=') {

//             let parsed_time = match time.parse::<i64>() {
//                 Ok(parsed_time) => parsed_time,
//                 Err(_) => {
//                     eprintln!("Invalid time value: {time}");
//                     return None
//                 }
//             };

//             timepoint.insert(
//                 rerun::Timeline::new_temporal(timeline_name),
//                 parsed_time,
//             );
//         }
//     }

//     for seq_str in &args.sequence {

        
//         if let Some((seqline_name, seq)) = seq_str.split_once('=') {

//             let parsed_time = match seq.parse::<i64>() {
//                 Ok(parsed_time) => parsed_time,
//                 Err(_) => {
//                     eprintln!("Invalid time value: {seq}");
//                     return None
//                 }
//             };
            
//             timepoint.insert(
//                 rerun::Timeline::new_sequence(seqline_name),
//                 parsed_time,
//             );
//         }
//     }

//     Some(timepoint)
// }
