// Copyright (C) 2019  Frank Rehberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>

use omg_idl_code_gen::{generate_with_search_path, Configuration};
use std::{
    env,
    fs::File,
    io::{Error, ErrorKind},
    path::Path,
};

const IDL_DIR: &str = "files";
const IDL_INFILE: &str = "dds/DdsCollection.idl";
const RUST_OUTFILE: &str = "DdsCollection.rs";

fn main() -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(RUST_OUTFILE);
    let config = Configuration::new(Path::new(IDL_DIR), Path::new(IDL_INFILE), false);
    let mut out = File::create(dest_path)?;

    generate_with_search_path(&mut out, &config).map_err(|_| Error::from(ErrorKind::NotFound))
}
