
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
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

    let (set, i) = search(FILE_PATH).unwrap();
    println!("Files analyzed : {}", i);

    // Now iterate and copy
    for path in set {        
        let val = format!("{}", get_extension_from_filename(&path).unwrap_or_else(|| "N/A"));
        copy(&path, TO_PATH_MAIN, &val).unwrap();        
    }

    Ok(())
}

fn search(path: &str) -> io::Result<(HashSet<PathBuf>, u32)> {
    use regex::{Regex};
    use lazy_static::lazy_static;

    let mut set = HashSet::new();
    let mut i = 0;


    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)canon|sony").unwrap();
    }

    for entry in fs::read_dir(path)? {
        i += 1;
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;
        if metadata.file_type().is_file() {
            let val = format!("{}", get_extension_from_filename(&path).unwrap_or_else(|| "N/A"));
            if allowed_type(&val) {
                // Now check to see if the image is from a Canon or Sony
                // Lots of sloppy unwraps, but eh .. fails it fails
                let file = File::open(path.to_str().unwrap()).unwrap();
                let exifResult = Reader::new().read_from_container(
                    &mut BufReader::new(&file));
                match exifResult {
                    Ok(exif) => {
                        if let Some(field) = get_string(&exif, Tag::Make) {                    
                            if RE.is_match(&field.to_lowercase()) {
                                set.insert(path);
                            }
                        }                
                    },
                    Err(e) => {
                        // PNG and a few i dont care about cant be processed by this file
                        // mostly it was JPG and CR2
                    }
                }
            }
        } 
        else if metadata.file_type().is_dir() {
            match search(&path.to_str().unwrap()) {
                Ok(ret) => {
                    let subset = ret.0;
                    let x = ret.1;
                    set.extend(subset);
                    i += x;
                },
                Err(err) => {
                    println!("Error : {:?} / {}", path, err);
                }
            }
        }
    }

    return Ok((set, i));
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

fn copy(full: &PathBuf, output_path: &str, extension: &str) -> io::Result<()> {
    let name = full.file_stem().unwrap().to_str().unwrap();
        
    let mut full_path = format!("{}/{}.{}", output_path, name, extension);
    let mut path = Path::new(full_path.as_str());
    if path.exists() {
        // add a random number to the front
        full_path = format!("{}/{}_{}.{}", output_path, random(), name, extension);
        path = Path::new(full_path.as_str());
    }
    
    std::fs::copy(full, path).unwrap();
    Ok(())
}

fn random() -> String {         // <5>
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

fn moved_type(s : &str) -> bool {
    match s.to_lowercase().as_str() {
        "mov" => true,
        "png" => true,
        "gif" => true,
        "m4p" => true,
        "jpg" => true,
        "jpeg"=> true,
        "cr2" => true,
        "tif" => true,
        "heic" => true,
        "mp3" => true,
        "mp4" => true,
        _ => false
    }
}