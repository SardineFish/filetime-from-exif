use std::{
    env,
    fs::{self, DirEntry},
    io,
    path::Path,
};

use chrono::{Datelike, NaiveDateTime};
use env_logger::Env;
use filetime::FileTime;
use rexif::ExifTag;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let dir = env::args()
        .nth(1)
        .unwrap_or_else(|| env::current_dir().unwrap().to_string_lossy().to_string());

    handle_recursive(dir);
}

fn handle_recursive(path: impl AsRef<Path>) {
    match fs::read_dir(path) {
        Err(err) => {
            log::error!("{err}");
        }
        Ok(read_dir) => {
            for entry in read_dir {
                if let Err(err) = handle_entry(entry) {
                    log::error!("{err}");
                }
            }
        }
    }
}

fn handle_entry(dir_entry: io::Result<DirEntry>) -> io::Result<()> {
    let file_entry = dir_entry?;
    if file_entry.file_type()?.is_dir() {
        handle_recursive(file_entry.path());
    } else if file_entry.file_type()?.is_file() {
        match rexif::parse_file(file_entry.path()) {
            Ok(data) => {
                if let Some(entry) = data
                    .entries
                    .iter()
                    .find(|entry| entry.tag == ExifTag::DateTime)
                {
                    // println!("{:?} - {}", entry.value, entry.value_more_readable);
                    match NaiveDateTime::parse_from_str(
                        &entry.value_more_readable,
                        "%Y:%m:%d %H:%M:%S",
                    ) {
                        Ok(time) if time.year() > 2000 => {
                            if let Err(err) = filetime::set_file_mtime(
                                file_entry.path(),
                                FileTime::from_unix_time(time.timestamp(), 0),
                            ) {
                                log::error!("{}", err);
                            } else {
                                log::info!("{:?} set mtime to {}", file_entry.path(), time);
                            }
                        }
                        Ok(time) => {
                            log::warn!(
                                "{:?} EXIF date time may be wrong: {}",
                                file_entry.path(),
                                time
                            );
                        }
                        Err(err) => {
                            log::error!("{err}");
                        }
                    }
                }
            }
            Err(_) => {
                log::warn!("{:?} Parse exif failed, ignore.", file_entry.path());
            }
        }
    }

    Ok(())
}
