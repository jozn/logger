use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

// Include the image file as a binary blob
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");

fn main() {

    write_file();

    // file.



    // Open the image with the default program
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    let status = Command::new("cmd")
        .args(&["/C", "start", "./my.jpg"])
        .status();
        // .expect("Failed to execute command");
    // assert!(status.success());

    // Get the current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Print the current directory
    println!("Current directory: {}", current_dir.display());

    // Open the image with the default program
    let status = Command::new("start")
        .arg("./my.jpg")
        .status();
        // .expect("Failed to execute command");
    // assert!(status.success());

    // Open the image with the default program
    match Command::new("start")
        .arg("./my.jpg")
        .status() {
        Ok(status) => {
            if status.success() {
                println!("Successfully opened the image.");
            } else {
                println!("Command executed, but reported failure.");
            }
        },
        Err(e) => println!("Failed to execute command: {}", e),
    };

    // Open the image with the default program
    match Command::new("cmd")
        .args(&["/C", "start", "./my.jpg"])
        .status() {
        Ok(status) => {
            if status.success() {
                println!("Successfully opened the image.");
            } else {
                println!("Command executed, but reported failure.");
            }
        },
        Err(e) => println!("Failed to execute command 222: {}", e),
    };

    // sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(50));
}

fn write_file() {
    // Write the image to a file
    let mut file = match File::create("./my.jpg") {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create file: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(IMAGE_DATA) {
        println!("Failed to write to file: {}", e);
        return;
    }

    // Use the 'attrib' command to hide the file
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    match Command::new("cmd")
        .args(&["/C", "attrib", "+H", "./my.jpg"])
        .status() {
        Ok(status) => {
            if !status.success() {
                println!("Command executed, but reported failure.");
            }
        },
        Err(e) => println!("Failed to execute command: {}", e),
    };
}

fn write_file2() {
    // Write the image to a file
    let mut file = File::create("./my.jpg").unwrap();
    file.write_all(IMAGE_DATA).unwrap();

    // Use the 'attrib' command to hide the file
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    let status = Command::new("cmd")
        .args(&["/C", "attrib", "+H", "./my.jpg"])
        .status();
        // .expect("Failed to execute command");
    // assert!(status.success());
}