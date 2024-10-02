use rfd::FileDialog;
use std::fs;
use std::io;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"\nPress Enter to continue...").unwrap();
    stdout.flush().unwrap();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Failed to read input!");
}

fn emails_with_keyword(dir: &Path, keyword: &str) -> Vec<PathBuf> {
    let start_time = Instant::now();
    let mut matching_keyword_email_paths = Vec::new();

    println!("Started the search...");

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if let Some(extension) = entry.path().extension() {
            if extension == "eml" {
                let email_path = entry.path().to_path_buf();
                if let Ok(mut file) = fs::File::open(&email_path) {
                    let mut contents = String::new();
                    println!("{}", contents);
                    if file.read_to_string(&mut contents).is_ok() {
                        if contents.contains(keyword) {
                            // Check if the email content contains keyword
                            matching_keyword_email_paths.push(email_path.clone());
                            println!("Found matching email at {}", email_path.display());
                        } else {
                            println!("No matching keyword at {}", email_path.display());
                        }
                    }
                }
            }
        }
    }

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!("Search took {:?} to complete", elapsed_time);

    matching_keyword_email_paths
}

fn copy_source_to_destination(source: &Path, destination: &Path) -> io::Result<()> {
    let start_time = Instant::now();

    println!("Started the copying of files...");

    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = destination.join(entry.file_name());

            if entry_path.is_dir() {
                copy_source_to_destination(&entry_path, &dest_path)?;
            } else {
                fs::copy(&entry_path, &dest_path)?;
            }
        }
    } else {
        fs::create_dir_all(destination.parent().unwrap())?;
        fs::copy(source, destination)?;
    }
    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!("Copy took {:?} to complete", elapsed_time);

    Ok(())
}

fn select_directory_via_gui(title: &str) -> PathBuf {
    let selected_dir = FileDialog::new()
        .set_title(title) // Set the window title
        .set_directory(".") // Optionally set a default directory
        .pick_folder()
        .expect("No directory selected!");

    selected_dir
}

fn main() {
    println!(r" ______                 _ _    _____                     _     ");
    println!(r"|  ____|               (_) |  / ____|                   | |    ");
    println!(r"| |__   _ __ ___   __ _ _| | | (___   ___  __ _ _ __ ___| |__  ");
    println!(r"|  __| | '_ ` _ \ / _` | | |  \___ \ / _ \/ _` | '__/ __| '_ \ ");
    println!(r"| |____| | | | | | (_| | | |  ____) |  __/ (_| | | | (__| | | |");
    println!(r"|______|_| |_| |_|\__,_|_|_| |_____/ \___|\__,_|_|  \___|_| |_|");
    println!("=================== Written by: Devon Casey ====================");
    println!(
        "This program will first prompt you for the 'root' path the .eml export(s) \
             from MailStore are located at."
    );
    println!(
        "The next file selector GUI will be for the destination that you want \
             the emails with the matching keyword to be exported to."
    );

    pause();

    // Prompt the user to select the search directory
    let search_directory = select_directory_via_gui("Select Source Directory");

    // Prompt the user to select the destination directory
    let copy_to_directory = select_directory_via_gui("Select Destination Directory");

    // Prompt the user to enter the keyword to search for.
    println!("Please enter search keyword: ");
    io::stdout().flush().unwrap();
    let mut keyword_to_search = String::new();
    io::stdin()
        .read_line(&mut keyword_to_search)
        .expect("Failed to read input!");
    let keyword_to_search = keyword_to_search.trim();

    // Call emails_with_keyword and return a vector of paths that match the entered keyword.
    let matching_paths = emails_with_keyword(&search_directory, keyword_to_search);

    // Iterate through matching_paths and copy them to the destination
    for matching_path in matching_paths {
        // Calculate the relative path from the search directory to the current matching file
        let relative_path = matching_path.strip_prefix(&search_directory).unwrap();
        let destination_path = copy_to_directory.join(relative_path);

        match copy_source_to_destination(&matching_path, &destination_path) {
            Ok(_) => println!("Successfully copied to {}", destination_path.display()),
            Err(e) => eprintln!("Failed to copy {}: {}", matching_path.display(), e),
        }
    }

    pause();
}
