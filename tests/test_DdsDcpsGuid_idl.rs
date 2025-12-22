extern crate rtps_idl;

use rtps_idl::{Configuration, generate_with_search_path};
use std::{
    io::{Error, ErrorKind, stdout},
    path::Path,
};

const IDL_DIR: &str = "crates/rtps-elements/files";
const IDL_INFILE: &str = "dds/DdsDcpsGuid.idl";

#[test]
fn convert_idl() -> Result<(), Error> {
    let config = Configuration::new(Path::new(IDL_DIR), Path::new(IDL_INFILE), false);

    generate_with_search_path(&mut stdout(), &config).map_err(|_| Error::from(ErrorKind::NotFound))
}
