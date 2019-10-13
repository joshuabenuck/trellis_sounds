use clap::{App, Arg};
use dirs;
use failure::{err_msg, Error};
use reqwest;
use rodio::{Device, Sink, Source};
use std::fs::{self, DirEntry, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::{thread, time};
use zip;

struct Pack {
    name: String,
    base: PathBuf,
    sounds: Vec<PathBuf>,
}

impl Pack {
    fn play(&self, sink: &Sink) {
        println!("Playing: {}", &self.name);
        for path in &self.sounds {
            println!("\t{}", &path.display());
            let file = File::open(self.base.join(&path)).unwrap();
            let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
            sink.append(source);
            sink.sleep_until_end();
        }
    }
}

struct Packs {
    packs: Vec<Pack>,
}

impl Packs {
    fn find_packs(base: &PathBuf) -> io::Result<Vec<Pack>> {
        let mut packs = Vec::<Pack>::new();
        let mut files = Vec::<PathBuf>::new();
        for entry in fs::read_dir(&base)? {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    packs.append(&mut Packs::find_packs(&base.join(&entry.file_name()))?);
                    continue;
                }
                if let Some(ext) = entry.path().extension() {
                    if ext != "wav" {
                        continue;
                    }
                    files.push(entry.path());
                }
            }
        }
        // Naming convention: a collection of packs has "packs" in its name.
        // Which means we want to skip them.
        if !&base
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .contains("packs")
        {
            // Detecting packs with more or less than four sounds may
            // eventually be needed here.
            packs.push(Pack {
                name: base.file_name().unwrap().to_str().unwrap().to_owned(),
                base: base.to_owned(),
                sounds: files,
            })
        }
        Ok(packs)
    }

    fn retrieve(url: &str, path: &PathBuf) -> io::Result<()> {
        let mut resp = reqwest::get(url).unwrap();
        assert!(resp.status().is_success());
        let mut buffer = Vec::new();
        resp.read_to_end(&mut buffer)?;
        fs::write(&path, buffer)?;
        Ok(())
    }

    fn unzip(fname: &PathBuf) {
        // Example code from zip crate's documentation
        let file = fs::File::open(&fname).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = fname.parent().unwrap().join(file.sanitized_name());

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {} comment: {}", i, comment);
                }
            }

            if (&*file.name()).ends_with('/') {
                println!(
                    "File {} extracted to \"{}\"",
                    i,
                    outpath.as_path().display()
                );
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.as_path().display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
    }

    fn from_dir(dir: &PathBuf) -> io::Result<Packs> {
        if !dir.exists() {
            eprintln!("Config dir does not exist.");
            eprintln!("Creating {}", dir.display());
            std::fs::create_dir(dir)?;
        }

        let target = &dir.join("sound_packs.zip");
        if !target.exists() {
            eprintln!("Downloading {}", &target.display());
            Packs::retrieve(
                "https://cdn-learn.adafruit.com/assets/assets/000/066/017/original/sound_packs.zip?1542348728",
                &target
                )?;
        }
        let packs_dir = dir.join("sound_packs");
        if !packs_dir.exists() {
            eprintln!("Unzipping {}", &target.display());
            Packs::unzip(&target);
        }
        Ok(Packs {
            packs: Packs::find_packs(&packs_dir)?,
        })
    }
}

fn starting_point(sink: &rodio::Sink, base: &PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(base)? {
        if let Ok(entry) = entry {
            let path = base.join(entry.file_name());
            println!("Processing {}", &path.display());
            if entry.path().is_dir() {
                starting_point(&sink, &path)?;
                continue;
            }
            if let Some(ext) = path.extension() {
                if ext != "wav" {
                    continue;
                }
                println!("Playing {}", &entry.file_name().to_str().unwrap());
                let file = File::open(&path).unwrap();
                let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                sink.append(source);
                sink.sleep_until_end();
                //thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
    Ok(())
}

fn process_packs<F>(base: &PathBuf, f: F)
where
    F: Fn(&Pack),
{
    match Packs::from_dir(&base.to_path_buf()) {
        Ok(packs) => {
            for pack in packs.packs {
                f(&pack);
                //pack.play(&sink);
            }
        }
        Err(err) => panic!(err),
    }
}

fn run() -> Result<(), Error> {
    let matches = App::new("sounds")
        .about("Utility to manage sound packs for the Adafruit M4 Trellis.")
        .arg(
            Arg::with_name("list")
                .long("list")
                .short("l")
                .help("List all sound packs."),
        )
        .arg(
            Arg::with_name("play")
                .long("play")
                .short("p")
                .takes_value(true)
                .default_value("all")
                .help("Play a sound pack."),
        )
        .get_matches();
    let home_dir = dirs::home_dir().ok_or(err_msg("Unable to find home directory!"))?;
    let base = home_dir.join(".trellis_sounds");
    if matches.is_present("list") {
        process_packs(&base.to_path_buf(), |pack| println!("{}", &pack.name));
        return Ok(());
    }
    if let Some(pack_name) = matches.value_of("play") {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);
        if pack_name == "all" {
            process_packs(&base.to_path_buf(), |pack| pack.play(&sink));
            return Ok(());
        }

        process_packs(&base.to_path_buf(), |pack| {
            if pack.name == pack_name {
                pack.play(&sink);
            }
        });
    }
    Ok(())
    /*
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    match Packs::from_dir(&base.to_path_buf()) {
        Ok(packs) => {
            for pack in packs.packs {
                //pack.play(&sink);
                println!("{}", pack.name);
            }
        }
        Err(err) => panic!(err),
    }
    */
    /*
    if let Err(err) = run(&sink, &base.to_path_buf()) {
        panic!(err);
    }
    sink.sleep_until_end();
    */
    /*
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        return 1;
    }
    let fname = std::path::Path::new(&*args[1]);
    let file = fs::File::open(&fname).unwrap();

    */
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
    }
}
