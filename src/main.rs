use rayon::prelude::*;
use rfd::FileDialog;
use std::fs::{self, OpenOptions};
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::time::SystemTime;
use walkdir::WalkDir;

fn log_message(message: &str) {
    let sys_time = SystemTime::now();
    let log_file_path = Path::new("C:\\Logs\\email_search.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Failed to open log file");

    writeln!(file, "{:?} {}", sys_time, message).expect("Failed to write to log file");
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"\nPress Enter to continue...").unwrap();
    stdout.flush().unwrap();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Failed to read input!");
}

fn select_directory_via_gui(title: &str) -> PathBuf {
    let selected_dir = FileDialog::new()
        .set_title(title) // Set the window title
        .set_directory(".") // Optionally set a default directory
        .pick_folder()
        .expect("No directory selected!");

    selected_dir
}

//noinspection ALL
fn find_emails_with_keyword_and_copy(dir: &Path, keyword: &str, copy_to_directory: &Path) {
    let start_time = Instant::now();
    let search_term = keyword.to_lowercase();
    let log_mutex = Arc::new(Mutex::new(()));

    println!("Started the search...");
    log_message("Started the search...");

    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .par_bridge() // RR seems to think this isn't loaded...
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("eml"))
        .for_each(|entry| {
            let email_path = entry.path().to_path_buf();
            if let Ok(mut file) = fs::File::open(&email_path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok()
                    && contents.to_lowercase().contains(&search_term)
                {
                    // Reads the emails contents as a string and converts it to lowercase.
                    let relative_path = email_path.strip_prefix(&dir).unwrap();
                    let destination_path = copy_to_directory.join(relative_path);
                    println!("Found matching email at {}", email_path.display());
                    log_message(&format!("Found matching email at {}", email_path.display()));
                    match copy_source_to_destination(&email_path, &destination_path) {
                        Ok(_) => {
                            let log_mutex = log_mutex.clone();
                            drop(log_mutex.lock().unwrap());
                            log_message(&format!(
                                "Successfully copied to {}",
                                destination_path.display()
                            ));
                        }
                        Err(e) => {
                            let log_mutex = log_mutex.clone();
                            drop(log_mutex.lock().unwrap());
                            log_message(&format!("Failed to copy {}: {}", email_path.display(), e));
                        }
                    }
                } else {
                    println!("No matching email found at {}", email_path.display());
                    log_message(&format!("No matching keyword at {}", email_path.display()));
                }
            }
        });

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!("Search and copy took {:?} to complete", elapsed_time);
    log_message(&format!(
        "Search and copy took {:?} to complete",
        elapsed_time
    ));
}

fn main() {
    println!(r" ______                 _ _    _____                     _     ");
    println!(r"|  ____|               (_) |  / ____|                   | |    ");
    println!(r"| |__   _ __ ___   __ _ _| | | (___   ___  __ _ _ __ ___| |__  ");
    println!(r"|  __| | '_ ` _ \ / _` | | |  \___ \ / _ \/ _` | '__/ __| '_ \ ");
    println!(r"| |____| | | | | | (_| | | |  ____) |  __/ (_| | | | (__| | | |");
    println!(r"|______|_| |_| |_|\__,_|_|_| |_____/ \___|\__,_|_|  \___|_| |_|");
    println!("=================== Written by: Devon Casey ====================");
    println!("This program will first prompt you for the 'root' path the .eml export(s) from MailStore are located at.");
    println!("The next file selector GUI will be for the destination that you want the emails with the matching keyword to be exported to.");

    pause();

    // Prompt the user to select the search directory
    let search_directory = select_directory_via_gui("Select Source Directory");

    // Prompt the user to select the destination directory
    let copy_to_directory = select_directory_via_gui("Select Destination Directory");

    // Prompt the user to enter the keyword to search for.
    println!("Please enter search keyword: ");
    stdout().flush().unwrap();
    let mut keyword_to_search = String::new();
    stdin()
        .read_line(&mut keyword_to_search)
        .expect("Failed to read input!");
    let keyword_to_search = keyword_to_search.trim();

    // Perform search and copy as emails with matching keywords are found
    find_emails_with_keyword_and_copy(&search_directory, keyword_to_search, &copy_to_directory);

    pause();
}
