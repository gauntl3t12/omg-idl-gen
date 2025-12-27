// Copyright (C) 2019  Frank Rehberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>

extern crate rtps_idl;

use std::{
    env,
    fs::File,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};
use rtps_idl::{Configuration, generate_with_search_path};

// TODO: improve the generator and run over ../../files/dds/DdsDcpsDomain.idl
const IDL_DIR: &str = "files";
const IDL_INFILE: &str = "dds/DdsCollection.idl";
const RUST_OUTFILE: &str = "DdsCollection.rs";

fn main() -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(RUST_OUTFILE);
    let mut config = Configuration::default();
    let mut out = File::create(dest_path)?;

    config.search_path = PathBuf::from(IDL_DIR);
    config.idl_file = PathBuf::from(IDL_INFILE);

    generate_with_search_path(&mut out, &config)
        .map_err(|_| Error::from(ErrorKind::NotFound))
}