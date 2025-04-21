use std::{
    fs,
    io::{self, Write},
    path::Path,
    thread, time,
};

use enigo::{Enigo, Keyboard, Settings};
use sheet::{Sheet, TokenDurations};

mod sheet;

fn play_sheet(enigo: &mut Enigo, music: Sheet, durations: &TokenDurations) {
    println!(
        "Playing '{}' by {}",
        music.header.title.unwrap_or(String::from("Unknown")),
        music.header.writer.unwrap_or(String::from("Unknown"))
    );
    println!("Starting in 5 seconds...");
    thread::sleep(time::Duration::from_secs(5));

    for token in music.tokens {
        match token {
            sheet::Token::Single(key) => {
                _ = enigo.key(key, enigo::Direction::Press);
                thread::sleep(time::Duration::from_secs_f64(durations.single));
                _ = enigo.key(key, enigo::Direction::Release);
            }
            sheet::Token::ShortPause => {
                thread::sleep(time::Duration::from_secs_f64(durations.short_pause))
            }
            sheet::Token::Pause => thread::sleep(time::Duration::from_secs_f64(durations.pause)),
            sheet::Token::LongPause => {
                thread::sleep(time::Duration::from_secs_f64(durations.long_pause))
            }
            sheet::Token::Many(keys) => {
                for key in &keys {
                    _ = enigo.key(*key, enigo::Direction::Press);
                }
                thread::sleep(time::Duration::from_secs_f64(durations.single));
                for key in keys {
                    _ = enigo.key(key, enigo::Direction::Release);
                }
            }
            sheet::Token::ManyFast(keys) => {
                for key in keys {
                    _ = enigo.key(key, enigo::Direction::Press);
                    thread::sleep(time::Duration::from_secs_f64(durations.many_fast));
                    _ = enigo.key(key, enigo::Direction::Release);
                }
            }
        }
    }
}

fn main() {
    let pause_distribution = sheet::PauseDistribution {
        short: 0.2,
        standard: 0.3,
        long: 0.5,
        pause_ratio: 20.0,
        many_fast_proportion: 0.15,
    };

    let sheets_dir = Path::new("./sheets"); // Path to your sheets directory

    let mut songs = Vec::new();
    if let Ok(entries) = fs::read_dir(sheets_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let file_contents = fs::read_to_string(path).unwrap();

                    songs.push(sheet::parse_sheet(file_contents.as_str()).unwrap());
                }
            }
        }
    } else {
        eprintln!("Error: Could not read the 'sheets' directory.");
        return;
    }

    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    loop {
        println!("\nSong Selection Menu:");
        if songs.is_empty() {
            println!("No songs found in the 'sheets' directory.");
            break;
        }
        for (i, song) in songs.iter().enumerate() {
            println!(
                "{}. '{}' by {}",
                i + 1,
                song.header.title.clone().unwrap_or("Unknown".to_string()),
                song.header.writer.clone().unwrap_or("Unknown".to_string())
            );
        }
        println!("{}. Exit", songs.len() + 1);

        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let choice: usize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input. Please enter a number.");
                continue;
            }
        };

        if choice == songs.len() + 1 {
            break;
        }

        if choice > 0 && choice <= songs.len() {
            let song = songs[choice - 1].clone();
            let durations = sheet::calculate_token_durations(
                song.header.length / song.tokens.iter().count() as f64,
                &pause_distribution,
            )
            .unwrap();

            println!("{:#?}", durations);
            play_sheet(&mut enigo, song, &durations);
        } else {
            println!("Invalid choice. Please try again.");
        }
    }
}
