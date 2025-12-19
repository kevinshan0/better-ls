use clap::Parser;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files or directories to list (defaults to current directory)
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Do not ignore entries starting with .
    #[arg(short, long)]
    all: bool,

    /// Use a long listing format
    #[arg(short, long)]
    long: bool,

    /// With -l, print sizes like 1K 234M 2G etc.
    #[arg(short = 'H', long)]
    human_readable: bool,
}

fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    // Type (simplified)
    s.push(if mode & 0o170000 == 0o040000 { 'd' } else { '-' });
    // Owner
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    // Group
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    // Others
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}

fn print_entry(path: &PathBuf, name: &str, args: &Args) {
    if args.long {
        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.permissions().mode();
            let size = metadata.len();
            let mode_str = format_mode(mode);
            
            // Simplified time formatting for now
            let _modified: SystemTime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            
            println!("{} {:>8} {}", mode_str, size, name);
        } else {
            println!("?????????? ???????? {}", name);
        }
    } else {
        println!("{}", name);
    }
}

fn main() {
    let args = Args::parse();

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    let num_paths = paths.len();

    for (index, path) in paths.iter().enumerate() {
        if num_paths > 1 {
            println!("{}:", path.display());
        }

        if path.is_dir() {
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let mut names: Vec<String> = Vec::new();
                    
                    if args.all {
                        names.push(".".to_string());
                        names.push("..".to_string());
                    }

                    for entry in entries {
                        if let Ok(entry) = entry {
                            let file_name = entry.file_name();
                            let name = file_name.to_string_lossy().to_string();

                            if !args.all && name.starts_with('.') {
                                continue;
                            }
                            names.push(name);
                        }
                    }
                    
                    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

                    for name in names {
                        let mut entry_path = path.clone();
                        if name != "." && name != ".." {
                            entry_path.push(&name);
                        }
                        print_entry(&entry_path, &name, &args);
                    }
                }
                Err(e) => eprintln!("{}: {}", path.display(), e),
            }
        } else {
            print_entry(&path, &path.to_string_lossy(), &args);
        }

        if index < num_paths - 1 {
            println!();
        }
    }
}