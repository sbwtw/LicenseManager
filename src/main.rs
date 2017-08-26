
extern crate clap;

use clap::{App, Arg};

use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::fs;
use std::fs::{File, ReadDir, OpenOptions};
use std::path::{Path, PathBuf};
use std::iter::Iterator;

struct Filter {
    base_dir: String,
    entry_iter: Option<ReadDir>,
    search_dirs: Vec<String>,
    ignore_dirs: Vec<String>,
    match_exts: Vec<String>,
}

impl Filter {
    pub fn new(path: &str) -> Filter {
        let f = Filter {
            base_dir: path.to_owned(),
            entry_iter: None,
            search_dirs: vec![path.to_owned()],
            ignore_dirs: vec![],
            match_exts: vec![],
        };

        f
    }

    pub fn ignore(mut self, paths: &Vec<&str>) -> Self {

        for p in paths {
            self.ignore_dirs.push((*p).to_owned());
        }

        self
    }

    pub fn extension(mut self, extensions: &Vec<&str>) -> Self {

        for e in extensions {
            self.match_exts.push((*e).to_owned());
        }

        self
    }
}

impl Iterator for Filter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {

        let base_len = self.base_dir.len() + 1;

        loop {
            if let Some(ref mut iter) = self.entry_iter {
                while let Some(r) = iter.next() {
                    let entry = r.unwrap();
                    let path = entry.path();
                    let metadata = entry.metadata().unwrap();

                    if metadata.is_file() {
                        if let Some(ext) = path.extension() {
                            if !self.match_exts.contains(
                                &ext.to_string_lossy().into_owned(),
                            )
                            {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        return Some(path);
                    } else if metadata.is_dir() {

                        let p = path.to_str().unwrap();
                        let tail = p.get(base_len..).unwrap();
                        if !self.ignore_dirs.contains(&tail.to_owned()) {
                            self.search_dirs.push(p.to_owned());
                        }
                    }
                }
            }

            if !self.search_dirs.is_empty() {
                let d = self.search_dirs.pop();

                if d.is_none() {
                    break;
                }

                self.entry_iter = Some(fs::read_dir(d.unwrap()).unwrap());
            } else {
                break;
            }
        }

        return None;
    }
}

struct Processor {
    license_text: String,
}

impl Processor {
    pub fn new() -> Processor {
        Processor { license_text: String::new() }
    }

    pub fn license(mut self, license: &str) -> Self {
        self.license_text = license.to_owned();
        self
    }

    pub fn has_license(&self, file: &str) -> bool {
        let f = File::open(file).unwrap();
        let rdr = BufReader::new(f);

        let mut scaned_lines = 0;
        for line in rdr.lines() {
            let l = line.unwrap();
            if l.contains("Copyright") {
                return true;
            }

            scaned_lines += 1;

            if scaned_lines > 5 {
                break;
            }
        }

        false
    }

    pub fn process(&self, file: &str) {
        if self.has_license(file) {
            return;
        }

        println!("process: {}", file);

        let mut buf = String::new();
        let mut f = File::open(file).unwrap();
        let _ = f.read_to_string(&mut buf);

        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file)
            .unwrap();

        f.write_all(self.license_text.as_bytes()).unwrap();
        f.write_all(buf.as_bytes()).unwrap();
    }
}

fn main() {

    let matches = App::new("license manager")
        .version("0.1")
        .about("license manager")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .help("specificed working directory")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let path = matches.value_of("path").unwrap();
    let path = Path::new(path).canonicalize().unwrap();
    let path = path.to_str().unwrap();

    let mut f = Filter::new(path).ignore(&vec![".git", "build"]).extension(
        &vec![
            "cpp",
            "h",
        ],
    );

    let license = r#"/**
 * Copyright (C) 2017 Deepin Technology Co., Ltd.
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3 of the License, or
 * (at your option) any later version.
 **/

"#;

    let p = Processor::new().license(license);

    while let Some(f) = f.next() {
        let file = f.to_str().unwrap();
        p.process(file);
    }
}
