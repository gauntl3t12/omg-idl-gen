use clap::{arg, command, value_parser, ArgAction};
use omg_idl_code_gen::{generate_with_search_path, Configuration};
use std::{
    fs::File,
    io::{stdout, Error, ErrorKind},
    path::PathBuf,
};

fn main() -> Result<(), std::io::Error> {
    let matches = command!()
    .arg(
        arg!(
            -I --include_dir <DIR> "Add the specified 'directory' to the search path for include files."
        )
        .default_value(".")
        .required(false)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
        arg!(
            -v --verbose ... "Turn verbose logging on"
        )
        .required(false)
        .action(ArgAction::SetTrue)
    )
    .arg(
        arg!(
            -o --output_file <FILE> "Write output to 'outfile'."
        )
        .required(false)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
        arg!(
            [idl_file] "IDL File to parse"
        )
        .required(true)
        .value_parser(value_parser!(PathBuf)),)
    .get_matches();

    let search_path = matches
        .get_one::<PathBuf>("include_dir")
        .expect("include_dir is defaulted");

    let idl_file = matches
        .get_one::<PathBuf>("idl_file")
        .expect("idl_file is required");

    let config = Configuration::new(search_path, idl_file, matches.get_flag("verbose"));

    let result = match matches.get_one::<PathBuf>("output_file") {
        Some(outfile) => {
            let mut of = File::create(outfile)?;
            generate_with_search_path(&mut of, &config)
        }
        _ => generate_with_search_path(&mut stdout(), &config),
    };

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            eprint!("parse error {:?}", err);
            Err(Error::new(ErrorKind::InvalidData, "parse error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use omg_idl_code_gen::{generate_with_search_path, Configuration};
    use std::{
        fs::File,
        io::{Cursor, Read},
        path::Path,
        str,
    };

    #[test]
    fn const_str() {
        testvector_verify("files/test-vectors/const_str/");
    }

    #[test]
    fn double_module_depth() {
        testvector_verify("files/test-vectors/double_module_depth/");
    }

    #[test]
    fn typedef_long() {
        testvector_verify("files/test-vectors/typedef_long/");
    }

    #[test]
    fn typedef_long_long() {
        testvector_verify("files/test-vectors/typedef_long_long/");
    }

    #[test]
    fn typedef_short() {
        testvector_verify("files/test-vectors/typedef_short/");
    }

    #[test]
    fn typedef_octet() {
        testvector_verify("files/test-vectors/typedef_octet/");
    }

    #[test]
    fn typedef_unsigned_short() {
        testvector_verify("files/test-vectors/typedef_unsigned_short/");
    }

    #[test]
    fn typedef_unsigned_long() {
        testvector_verify("files/test-vectors/typedef_unsigned_long");
    }

    #[test]
    fn typedef_unsigned_long_long() {
        testvector_verify("files/test-vectors/typedef_unsigned_long_long");
    }

    #[test]
    fn typedef_char() {
        testvector_verify("files/test-vectors/typedef_char");
    }

    #[test]
    fn typedef_wchar() {
        testvector_verify("files/test-vectors/typedef_wchar");
    }

    #[test]
    fn typedef_string() {
        testvector_verify("files/test-vectors/typedef_string");
    }

    #[test]
    fn typedef_wstring() {
        testvector_verify("files/test-vectors/typedef_wstring");
    }

    #[test]
    fn typedef_string_bounded() {
        testvector_verify("files/test-vectors/typedef_string_bounded");
    }

    #[test]
    fn typedef_wstring_bounded() {
        testvector_verify("files/test-vectors/typedef_wstring_bounded");
    }

    #[test]
    fn typedef_sequence() {
        testvector_verify("files/test-vectors/typedef_sequence");
    }

    #[test]
    fn typedef_array_dim_1() {
        testvector_verify("files/test-vectors/typedef_array_dim_1");
    }

    #[test]
    fn typedef_array_dim_2() {
        testvector_verify("files/test-vectors/typedef_array_dim_2");
    }

    #[test]
    fn struct_members() {
        testvector_verify("files/test-vectors/struct_members");
    }

    #[test]
    fn enum_variants() {
        testvector_verify("files/test-vectors/enum_variants");
    }

    #[test]
    fn struct_module() {
        testvector_verify("files/test-vectors/struct_module");
    }

    #[test]
    fn const_op_and() {
        testvector_verify("files/test-vectors/const_op_and");
    }

    #[test]
    fn const_op_add() {
        testvector_verify("files/test-vectors/const_op_add");
    }

    #[test]
    fn const_op_sub() {
        testvector_verify("files/test-vectors/const_op_sub");
    }

    #[test]
    fn const_op_lshift() {
        testvector_verify("files/test-vectors/const_op_lshift");
    }

    #[test]
    fn const_op_rshift() {
        testvector_verify("files/test-vectors/const_op_rshift");
    }

    #[test]
    fn const_op_or() {
        testvector_verify("files/test-vectors/const_op_or");
    }

    #[test]
    fn const_op_xor() {
        testvector_verify("files/test-vectors/const_op_xor");
    }

    #[test]
    fn const_op_mul() {
        testvector_verify("files/test-vectors/const_op_mul");
    }

    #[test]
    fn const_op_div() {
        testvector_verify("files/test-vectors/const_op_div");
    }

    #[test]
    fn const_op_mod() {
        testvector_verify("files/test-vectors/const_op_mod");
    }

    #[test]
    fn include_directive() {
        testvector_verify("files/test-vectors/include_directive/");
    }

    #[test]
    fn union_members() {
        testvector_verify("files/test-vectors/union_members");
    }

    fn testvector_verify(testvector: &str) {
        let expected_path = Path::new(testvector).join("expected.rs");

        let mut expected_file = match File::open(expected_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}", err);
                panic!();
            }
        };
        let mut expected = String::new();
        assert!(expected_file.read_to_string(&mut expected).is_ok());

        let config = Configuration::new(Path::new(testvector), Path::new("input.idl"), false);

        // Create fake "file"
        let mut out = Cursor::new(Vec::new());
        match generate_with_search_path(&mut out, &config) {
            Ok(_) => (),
            Err(err) => {
                eprint!("parse error {:?}", err);
                panic!();
            }
        };
        print_buffer(out.get_ref());
        let expected_no_carriage: Vec<u8> = expected
            .as_bytes()
            .iter()
            .filter(|&&b| b != b'\r')
            .copied()
            .collect();
        let text_no_carriage: Vec<u8> = out
            .get_ref()
            .iter()
            .filter(|&&b| b != b'\r')
            .copied()
            .collect();
        assert_eq!(expected_no_carriage.as_slice(), text_no_carriage.as_slice());
    }

    fn print_buffer(buf: &Vec<u8>) {
        let content = str::from_utf8(&buf).unwrap();

        println!("{}", content);
    }
}
