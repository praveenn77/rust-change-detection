use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::env;
use std::hash::Hasher;
use gxhash::GxHasher;
use std::time::Instant;

// Hash all the files in the directories
fn hash_files_in_directories<P: AsRef<Path>>(dirs: &[P], exclude_folders: &[&str], exclude_files: &[&str]) -> io::Result<HashMap<PathBuf, String>> {
    let mut hash_map = HashMap::new();
    for dir in dirs {
        hash_files_in_directory(dir, &mut hash_map, exclude_folders, exclude_files)?;
    }
    Ok(hash_map)
}

// Hash all the files in the directory and return a hash map of the file paths and their hashes
fn hash_files_in_directory<P: AsRef<Path>>(dir: P, hash_map: &mut HashMap<PathBuf, String>,  exclude_folders: &[&str], exclude_files: &[&str]) -> io::Result<()> {
    fn visit_dirs(dir: &Path, exclude_folders: &[&str], exclude_files: &[&str], cb: &mut dyn FnMut(&Path) -> io::Result<()>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if exclude_folders.iter().any(|&folder| path.ends_with(folder))  {
                        continue;
                    }
                    visit_dirs(&path, exclude_folders, exclude_files, cb)?;
                } else {
                    if exclude_files.iter().any(|&file| path.ends_with(file)) {
                        continue;
                    }
                    cb(&path)?;
                }
            }
        }
        Ok(())
    }
  
   
    visit_dirs(dir.as_ref(), exclude_folders, exclude_files, &mut |path| {
        let mut file_content = fs::File::open(path)?;
        let mut buffer = Vec::new();
        let mut hasher = GxHasher::with_seed(1234);
        file_content.read_to_end(&mut buffer)?;
        hasher.write(&buffer);
        let hash_string = hasher.finish();
        hash_map.insert(path.to_path_buf(), hash_string.to_string());
        Ok(())
    })?;
    Ok(())
}


// Read the existing hash results from the file
fn read_hashes_from_file(file_path: &Path) -> io::Result<HashMap<PathBuf, String>> {
    // let file = File::open(file_path)?;

    let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&file_path)?;

    // If the file does not exist, return an empty hash map
    if !std::path::Path::new(file_path).exists() {
        return Ok(HashMap::new());
    }

    let reader = BufReader::new(file);
    let mut hash_map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((path_str, hash_str)) = line.split_once(": ") {
            let path = PathBuf::from(path_str.trim_matches('"'));
            let hash = hash_str.trim_matches('"').to_string();
            hash_map.insert(path, hash);
        }
    }

    Ok(hash_map)
}

// Write the hash results to the file
fn write_hash_to_file(results: &HashMap<PathBuf, String>, output_file: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&output_file)?;

     // Clear the file
     file.set_len(0)?;

    for (path, hash) in results {
        writeln!(file, "{:?}: {:?}", path, hash)?;
    }
    Ok(())
}

fn write_vec_to_file(non_matching_keys: &[PathBuf], output_file: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&output_file)?;

    // Clear the file
    file.set_len(0)?;

    for key in non_matching_keys {
        writeln!(file, "{:?}", key)?;
    }
    Ok(())
}

fn compare_hash_maps(
    map1: &HashMap<PathBuf, String>,
    map2: &mut HashMap<PathBuf, String>,
    log_level: &str
) -> Vec<PathBuf> {
    let mut non_matching_keys = Vec::new();

    for key in map1.keys() {
        if map2.get(key) != map1.get(key) {
            non_matching_keys.push(key.clone());
            if log_level == "debug" {
                println!("{:?}: {:?}", key, "changed/added");
            }
        } 
        map2.remove(key);
    }

    for key in map2.keys() {
        non_matching_keys.push(key.clone());
        if log_level == "debug" {
            println!("{:?}: {:?}", key, "removed");
        }
    }

    non_matching_keys
}

fn main() -> io::Result<()> {
   
    let start = Instant::now();

    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());
  // Read the environment variable
    let file_paths = env::var("RCD_FOLDERS").expect("RCD_FOLDERS environment variable not set");

    let default_exclude_folders = "node_modules;dist;.git;coverage;.turbo";

    let exclude_folders_env = env::var("RCD_IGNORE_FOLDERS").unwrap_or(default_exclude_folders.to_string());

    let default_exclude_files = ".gitignore;.prettierrc;.eslintrc;.babelrc;.DS_Store;Thumbs.db";

    let exclude_files_env = env::var("RCD_IGNORE_FILES").unwrap_or(default_exclude_files.to_string());

    let default_hash_file = ".rcd_hash";

    let hash_file_env = env::var("RCD_HASH_FILE").unwrap_or(default_hash_file.to_string());

    let default_output_file = ".rcd_log";

    let output_file_env = env::var("RCD_LOG_FILE").unwrap_or(default_output_file.to_string());

    let log_level = env::var("RCD_LOG_LEVEL").unwrap_or("info".to_string());

    // Split the paths and collect them into a vector
    let directories: Vec<PathBuf> = file_paths.split(';').map(PathBuf::from).collect();

    // Split the exclude folders and collect them into a vector
    let mut exclude_folders: Vec<&str> = exclude_folders_env.split(';').collect();

    exclude_folders.push("node_modules");

    // Split the exclude files and collect them into a vector
    let exclude_files: Vec<&str> = exclude_files_env.split(';').collect();

    let hash_file = Path::new(&hash_file_env);

    let output_file = Path::new(&output_file_env);

    println!("Reading files from the directories: {:?}", directories);

    // Hash all the files in the directory
    let current_hash = hash_files_in_directories(&directories, &exclude_folders, &exclude_files)?;

    println!("Reading the existing hash results from the file: {:?}", hash_file);

    // Read the existing hash results from the file
    let mut existing_hash = read_hashes_from_file(hash_file)?;

    println!("Comparing the current hash results with the existing hash results");

    // Compare the current hash results with the existing hash results
    let non_matching_files = compare_hash_maps(&current_hash, &mut existing_hash, &log_level);

    println!("Writing the non-matching files to the file: {:?}", output_file);
    // Write the non-matching files to the file
    write_vec_to_file(&non_matching_files,  output_file)?;
    
    println!("Writing the current hash results to the file: {:?}", hash_file);
    // Update the current hash results to the file
    write_hash_to_file(&current_hash, hash_file)?;

    println!("Done!");

    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);

    Ok(())
}
