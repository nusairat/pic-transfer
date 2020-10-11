
use async_std::{task, fs};
use async_std::path::{Path, PathBuf};
use std::io;
use std::ffi::OsStr;
use std::io::BufReader;
use exif::{Exif, Reader, In, Tag};
use std::fs::File;

const FILE_PATH: &str = "/Volumes/External4TB/Recoveries";
// const FILE_PATH: &str = "/Volumes/External4TB/temp";
const TO_PATH_MAIN: &str = "/Users/joseph/Downloads/Processed/Extra";

fn main() -> io::Result<()> {
    println!("Anaylze Types!");

    process(FILE_PATH).unwrap();

    println!("DONE");

    Ok(())
}

fn process(path: &str) -> io::Result<()> {
    use regex::{Regex};
    use lazy_static::lazy_static;
    use async_std::prelude::*;


    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)canon|sony").unwrap();
    }

    task::block_on(async {
        let mut dir = fs::read_dir(&path).await?;

        while let Some(res) = dir.next().await {
            let entry = res?;
            let path = entry.path();

            let metadata = fs::metadata(&path).await?;
            if metadata.file_type().is_file() {
                let val = format!("{}", get_extension_from_filename(&path).unwrap_or_else(|| "N/A"));
                if allowed_type(&val) {
                    // Now check to see if the image is from a Canon or Sony
                    // Lots of sloppy unwraps, but eh .. fails it fails
                    let file = File::open(path.to_str().unwrap()).unwrap();
                    let exif_result = Reader::new().read_from_container(
                        &mut BufReader::new(&file));
                    match exif_result {
                        Ok(exif) => {
                            if let Some(field) = get_string(&exif, Tag::Make) {                    
                                if RE.is_match(&field.to_lowercase()) {
                                    //set.insert(path);
                                    println!("FOUND :: {:?}", path);
                                    copy(&path, TO_PATH_MAIN, &val).await.unwrap();
                                }
                            }                
                        },
                        Err(_) => {
                            // PNG and a few i dont care about cant be processed by this file
                            // mostly it was JPG and CR2
                        }
                    }
                }
            } 
            else if metadata.file_type().is_dir() {
                process(&path.to_str().unwrap()).unwrap();
            }
        }        
        Ok(())
    })    
}

fn get_string(reader: &Exif, tag: Tag) -> Option<String> {
    reader.get_field(tag,In::PRIMARY)
        .and_then(|field| Some(field.value.display_as(tag).to_string()) ) // <3>
}

fn get_extension_from_filename(path: &PathBuf) -> Option<&str> {
    path
        .extension()
        .and_then(OsStr::to_str)
}

async fn copy(full: &PathBuf, output_path: &str, extension: &str) -> io::Result<()> {
    let name = full.file_stem().unwrap().to_str().unwrap();
        
    let mut full_path = format!("{}/{}.{}", output_path, name, extension);
    let mut path = Path::new(full_path.as_str());

    if path.exists().await {
        // add a random number to the front
        full_path = format!("{}/{}_{}.{}", output_path, random(), name, extension);
        path = Path::new(full_path.as_str());
    }
    
    std::fs::copy(full, path)?;
    Ok(())
}

fn random() -> String {        
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .collect()
}

fn allowed_type(s : &str) -> bool {
    match s.to_lowercase().as_str() {
        "mov" => true,
        "m4p" => true,
        "jpg" => true,
        "jpeg"=> true,
        "cr2" => true,
        "tif" => true,
        "mp4" => true,
        _ => false
    }
}

// fn moved_type(s : &str) -> bool {
//     match s.to_lowercase().as_str() {
//         "mov" => true,
//         "png" => true,
//         "gif" => true,
//         "m4p" => true,
//         "jpg" => true,
//         "jpeg"=> true,
//         "cr2" => true,
//         "tif" => true,
//         "heic" => true,
//         "mp3" => true,
//         "mp4" => true,
//         _ => false
//     }
// }