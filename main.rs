use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::thread;
use csv::Reader;
use enigo::{Enigo, MouseButton, MouseControllable};
use serde::Deserialize;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct MouseAction {
    action: String,
    x_position: Option<i32>,
    y_position: Option<i32>,
    delay_ms: Option<u64>,
    button: Option<String>,
    modifiers: Option<String>,
    repeat_count: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Print current directory for debugging
    println!("Current directory: {:?}", env::current_dir()?);
    
    // Create mouse controller
    let mut enigo = Enigo::new();
    
    // Determine CSV file path with robust handling
    let csv_path = determine_csv_path()?;
    println!("Using CSV file: {}", csv_path);
    
    // Open and parse the CSV file
    let file = File::open(&csv_path)?;
    let mut reader = Reader::from_reader(file);
    
    println!("Successfully opened CSV file. Starting automation...");
    
    // Process each row in the CSV
    for result in reader.deserialize() {
        let record: MouseAction = result?;
        println!("Executing action: {:?}", record);
        
        // Apply delay if specified
        if let Some(delay) = record.delay_ms {
            thread::sleep(Duration::from_millis(delay));
        }
        
        // Get repeat count (default to 1)
        let repeat_count = record.repeat_count.unwrap_or(1);
        
        // Execute the action the specified number of times
        for _ in 0..repeat_count {
            match record.action.as_str() {
                "move" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Moving to position: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                    }
                },
                "move_relative" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Moving relatively by: ({}, {})", x, y);
                        enigo.mouse_move_relative(x, y);
                    }
                },
                "click" => {
                    // First move to position if specified
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Moving to position: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                    }
                    
                    // Then click with specified button (default to left)
                    let button = match record.button.as_deref() {
                        Some("right") => MouseButton::Right,
                        Some("middle") => MouseButton::Middle,
                        _ => MouseButton::Left,
                    };
                    
                    println!("Clicking with {:?} button", button);
                    enigo.mouse_click(button);
                },
                "double_click" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Moving to position: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                    }
                    
                    let button = match record.button.as_deref() {
                        Some("right") => MouseButton::Right,
                        Some("middle") => MouseButton::Middle,
                        _ => MouseButton::Left,
                    };
                    
                    println!("Double-clicking with {:?} button", button);
                    enigo.mouse_click(button);
                    thread::sleep(Duration::from_millis(10)); // Small delay between clicks
                    enigo.mouse_click(button);
                },
                "right_click" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Moving to position: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                    }
                    println!("Right-clicking");
                    enigo.mouse_click(MouseButton::Right);
                },
                "drag" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Starting drag at: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                        enigo.mouse_down(MouseButton::Left);
                    }
                },
                "release" => {
                    if let (Some(x), Some(y)) = (record.x_position, record.y_position) {
                        println!("Releasing at: ({}, {})", x, y);
                        enigo.mouse_move_to(x, y);
                    }
                    println!("Releasing mouse button");
                    enigo.mouse_up(MouseButton::Left);
                },
                "scroll" => {
                    let direction = match record.modifiers.as_deref() {
                        Some("down") => -1,
                        _ => 1,
                    };
                    
                    let amount = repeat_count as i32;
                    println!("Scrolling {} by {} units", if direction > 0 {"up"} else {"down"}, amount);
                    enigo.mouse_scroll_y(direction * amount);
                },
                "wait" => {
                    println!("Waiting...");
                    // Already handled by the delay logic
                },
                _ => {
                    println!("Unknown action: {}", record.action);
                }
            }
        }
    }
    
    println!("Automation completed successfully!");
    Ok(())
}

// Helper function to determine the CSV file path
fn determine_csv_path() -> Result<String, Box<dyn Error>> {
    // Check if path is provided as command line argument
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        if Path::new(path).exists() {
            return Ok(path.to_string());
        } else {
            println!("Warning: Specified file '{}' not found.", path);
            println!("Falling back to default locations...");
        }
    }
    
    // Create the default CSV file if it doesn't exist
    let default_csv_path = "mouse_actions.csv";
    if !Path::new(default_csv_path).exists() {
        println!("Default CSV file not found. Creating one at '{}'...", default_csv_path);
        
        // Create and write the default CSV content
        let csv_data = "action,x_position,y_position,delay_ms,button,modifiers,repeat_count\n\
                        move,100,200,500,,,\n\
                        click,150,300,200,left,,1\n\
                        double_click,150,300,150,left,,2\n\
                        right_click,400,500,300,right,,1\n\
                        drag,200,300,100,left,,\n\
                        release,400,500,50,,,\n\
                        move,500,600,300,,,\n\
                        scroll,500,600,200,,down,5\n\
                        wait,,,2000,,,\n\
                        move_relative,50,-30,300,,,";
        
        std::fs::write(default_csv_path, csv_data)?;
        println!("Default CSV file created successfully!");
    }
    
    // Try multiple possible locations
    let default_paths = vec![
        "mouse_actions.csv",          // In current directory
        "./mouse_actions.csv",        // Explicit current directory
        "../mouse_actions.csv",       // Parent directory
        "data/mouse_actions.csv",     // Data subdirectory
    ];
    
    for path in default_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    // If we get here, we've already created the default file, so it should exist
    Ok(default_csv_path.to_string())
}