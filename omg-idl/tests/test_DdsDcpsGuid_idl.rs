use omg_idl_code_gen::{Configuration, generate_with_search_path};
use std::{
    io::{Error, ErrorKind, stdout},
    path::{Path, PathBuf},
};

const IDL_DIR: &str = "omg-elements/files";
const IDL_INFILE: &str = "dds/DdsDcpsGuid.idl";

#[test]
fn convert_idl() -> Result<(), Error> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()?;
    let idl_dir = workspace_root.join(IDL_DIR);

    let config = Configuration::new(&idl_dir, Path::new(IDL_INFILE), false);
    generate_with_search_path(&mut stdout(), &config).map_err(|_| Error::from(ErrorKind::NotFound))
}
