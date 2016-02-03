
extern crate clap;
extern crate walkdir;

use clap::Arg;
use clap::App;

use walkdir::WalkDir;

use std::io;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;
use std::fs::OpenOptions;
use std::fs::File;
use std::thread;

static LICENSE: &'static str = "/**
 * Copyright (C) 2015 Deepin Technology Co., Ltd.
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3 of the License, or
 * (at your option) any later version.
 **/\n\n";

fn process_file(file: &Path) {

    let file_name = file.to_string_lossy();

    if !file_name.ends_with(".h") && !file_name.ends_with(".cpp") {
        return;
    }

    println!("process {:?}", file);


    // read old data
    let mut fp = File::open(file).unwrap();
    let mut buf = String::new();
    fp.read_to_string(&mut buf);

    // truncate file
    let mut fp = OpenOptions::new().write(true).truncate(true).open(file).unwrap();

    thread::spawn(move || {
        // write new data
        fp.write_all(LICENSE.as_bytes()).unwrap();
        fp.write_all(&buf.into_bytes()).unwrap();
        fp.sync_data();
    });
}

fn main() {

    let args = App::new("license_manage")
                    .version("0.0.1")
                    .author("sbwtw <sbw@sbw.so>")
                    .arg(Arg::with_name("path")
                         .short("p")
                         .long("path")
                         .help("search path")
                         .takes_value(true))
                    .get_matches();

    let search_path = match args.value_of("path") {
        Some(value) => value,
        _ => ".",
    };

    for entry in WalkDir::new(search_path) {

        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        let file_type = entry.file_type();

        if file_type.is_file() {
            process_file(entry.path());
        }
    }
}
