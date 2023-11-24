use base64::encode;
use clap::builder::Str;
use clap::Parser;
use json::JsonValue;
use json::object;
use json::object::Object;
use json::stringify;
use std::fs;
use std::io::Read;
use std::io::Write;
use uuid::Uuid;

#[derive(Parser)]
struct Cli {
    pattern: String,
    /// Title of commit
    #[arg(short, long, default_value_t)]
    title: String,
    #[arg(short, long, default_value_t)]
    path: String,
    #[arg(short, long, default_value_t)]
    ignore: String,
}

fn main() {
    let args: Cli = Cli::parse();
    //let paths = fs::read_dir("");
    if args.pattern == "config" {
        fs::create_dir(".rvscw");
        let content = object! {
            "ignore": format!(".rvscw {}", args.ignore),
        };
        let mut config = fs::File::create(".rvscw/ignore.rvscw").unwrap();
        config.write_all(stringify(content).as_bytes());
    };

    let ignore = read_ignore();
    if args.pattern == "init" {
        cli_init(ignore);
    } else if args.pattern == "status" {
        status(&ignore);
    } else if args.pattern == "commit" {
        commit(args.title, &ignore);
    } else if args.pattern == "back" {
        back(args.title);
    } else if args.pattern == "log" {
            log();
    };
}

// ignore
fn read_ignore() -> String {
    let mut content = fs::read_to_string(".rvscw/ignore.rvscw").unwrap();
    let parsed = json::parse(&content).unwrap();
    drop(content);
    return parsed["ignore"].to_string();
}

fn cli_init(ignore: String) {
    let real_path: &str = ".rvscw";

    let uuid = Uuid::new_v4();
    write_last(real_path.to_string(), uuid.to_string());
    new_log(uuid.to_string(), "init".to_string());

    let mut fsdata = String::new();
    let mut metadata = json::JsonValue::new_object();
    (fsdata, metadata[uuid.to_string()]) = new_rvscw_files("".to_string(), &ignore);

    // create new files fs=files meta=metdata
    // later this files will be used for detecting if there are some differences
    let mut fsjson = fs::File::create(format!("{}/cont", real_path)).expect("File: ");
    fsjson.write_all(format!("{}:::init::{}", uuid.to_string(), fsdata).as_bytes());
    drop(fsjson);
    drop(fsdata);
    drop(uuid);

    let mut metajson = fs::File::create(format!("{}/meta.json", real_path)).expect("File: ");
    metajson.write_all(stringify(metadata).as_bytes());
    drop(metajson);
}

// scan all folders
// read content and time of modification
fn new_rvscw_files(path: String, ignore: &String) -> (String, json::JsonValue) {
    let mut fscontent = String::new();
    let mut fsmetadata = json::JsonValue::new_object();
    for file in fs::read_dir(path).unwrap() {
        let mut filepath: &String = &file.unwrap().path().display().to_string();
        let metadata = fs::metadata(filepath).expect("Error: ");
        println!("{filepath}");

        if ignore.contains(filepath) {
            continue;
        }

        if metadata.is_file() {
            let content = fs::read(filepath).expect("Should not read: ");
            fscontent += &format!(
                "{}:{}:",
                filepath.to_string(),
                base64::encode(content).to_string()
            );
            fsmetadata[filepath] =
                object! {"mdftime": format!("{:?}", metadata.modified().unwrap())};
        // later need to change this and pass some .gitignore like thing
        } else if metadata.is_dir() {
            let (fordircont, fordirmeta) = new_rvscw_files(filepath.to_string(), ignore);
            fscontent += &fordircont;
            fsmetadata[filepath] = fordirmeta;
        }
    }

    return (fscontent, fsmetadata);
}

fn status(ignore: &String) {
    let last_id = read_last(".rvscw".to_string());
    let metajson = read_meta();
    let scanmeta = scan_for_meta("".to_string(), ignore);
    println!("Changed: ");
    compare(metajson[last_id].to_owned(), scanmeta);
}

fn compare(metajson: json::JsonValue, scanmeta: json::JsonValue) {
    for (key, _) in metajson.entries() {
        if metajson[key].len() > 1 {
            compare(metajson[key].to_owned(), scanmeta[key].to_owned());
            continue;
        }

        if metajson[key] != scanmeta[key] {
            println!("{}", key);
        }
    }
}

fn scan_for_meta(path: String, ignore: &String) -> json::JsonValue {
    let mut fsmetadata = json::JsonValue::new_object();
    for file in fs::read_dir(path).unwrap() {
        let mut filepath: &String = &file.unwrap().path().display().to_string();
        let metadata = fs::metadata(filepath).expect("Error: ");

        if ignore.contains(filepath) {
            continue;
        }

        if metadata.is_file() {
            let content = fs::read(filepath).expect("Should not read: ");
            fsmetadata[filepath] =
                object! {"mdftime": format!("{:?}", metadata.modified().unwrap())};
        } else if metadata.is_dir() {
            fsmetadata[filepath] = scan_for_meta(filepath.to_string(), ignore);
        }
    }

    return fsmetadata;
}

fn read_meta() -> json::JsonValue {
    let meta = fs::read_to_string(".rvscw/meta.json").expect("Should not read: ");
    let parsed = json::parse(&meta).unwrap();
    return parsed;
}

fn write_last(path: String, id: String) {
    let mut file = fs::File::create(format!("{}/last", path)).expect("Update: ");
    file.write_all(id.as_bytes());
    drop(file);
}

fn read_last(path: String) -> String {
    let mut file = fs::File::open(format!("{}/last", path)).expect("Update: ");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    return contents;
}

fn write_commits(path: String, id: String, title: String) {
    let mut file = fs::File::create(format!("{}/commits", path)).expect("Update: ");
    file.write_all(format!("{id} {title}").as_bytes());
    drop(file);
}

fn get_last_commit(path: String) -> String {
    let mut file = fs::File::open(format!("{}/last", path)).expect("Update: ");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    return contents;
}

fn commit(title: String, ignore: &String) {
    let path: String = ".rvscw".to_string();
    let uuid = Uuid::new_v4();
    let copy_title = &title;
    write_last(path.to_string(), uuid.to_string());
    write_log(uuid.to_string(), copy_title.to_string());

    let mut fsdata = String::new();
    let mut newmeta = json::JsonValue::new_object();
    (fsdata, newmeta) = new_rvscw_files("".to_string(), ignore);
    let mut files = fs::File::options()
        .append(true)
        .open(format!("{path}/cont"))
        .expect("Files: ");
    files
        .write(format!("\n{}:::{}::{}", uuid.to_string(), copy_title, fsdata).as_bytes())
        .expect("Files writing error: ");
    drop(files);

    let mut metadata = read_meta();
    metadata[uuid.to_string()] = newmeta;

    let mut metajson = fs::File::create(format!("{}/meta.json", path)).unwrap();
    metajson.write_all(stringify(metadata).as_bytes());

    write_last(path, uuid.to_string());
}

// back to the given title commit
fn back(commit_id: String,) {
    let mut files_content = fs::read_to_string(".rvscw/cont").unwrap();
    let mut aim: String = String::new();
    let id = &commit_id;
    write_last(".rvscw".to_string(), id.to_string());

    // finding aimed commit id
    for value in files_content.split("\n") {
        if value.contains(id) {
            aim = value.to_string();
            drop(id);
            break;  
        }
    }

    let mut i: i32 = 0;
    for section in aim.split("::") {
        if i < 1 {
            i += 1;
            continue;
        }
        
        files_content = section.to_string();
    }
    drop(aim);

    i = 0;
    let mut last_file: String = String::new();
    for file in files_content.split(":") {
        println!("file: {}", file);
        if i % 2 == 0 {
            last_file = file.to_string();
            i += 1;
            continue;
        }

        let mut new_file = fs::File::create(&last_file).unwrap();
        new_file.write_all(&base64::decode(file).unwrap());
        drop(new_file);

        i += 1;
    }
}

fn log() {
    let mut files_content = fs::read_to_string(".rvscw/log").unwrap();
    println!("Log: \n{}", files_content);
    drop(files_content);
}

fn new_log(id: String, title: String) {
    let mut file = fs::File::create(".rvscw/log").unwrap();
    file.write(format!("ID:{} Title:{}", id, title).as_bytes());
    drop(file);
}

fn write_log(id: String, title: String) {
    let mut file = fs::File::options()
    .append(true)
    .open(".rvscw/log")
    .unwrap();
    file.write(format!("\nID:{} Title:{}", id, title).as_bytes());
    drop(file);
}
