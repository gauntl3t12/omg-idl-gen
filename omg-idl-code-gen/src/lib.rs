// Copyright (C) 2025  Bryan Conn
// Copyright (C) 2019  Frank Rehberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>
mod ast;

use ast::*;
use omg_idl_grammar::{IdlParser, Rule};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::{
    fs::File,
    io::{Error, Read, Write},
    path::{Path, PathBuf},
};

/// TODO change to this_error
#[derive(Debug)]
pub enum IdlError {
    InternalError,
    UnexpectedItem(Rule),
    ExpectedItem(Rule),
    ErrorMesg(String),
    FileNotFound(String),
}

/// All IDL Loader must be capable of reading data into the system
pub trait IdlLoader {
    fn load(&self, filename: &Path) -> Result<String, Error>;
}

/// Container for where to find a file and if extra logging occur
#[derive(Debug, Default)]
pub struct Configuration {
    search_path: PathBuf,
    idl_file: PathBuf,
    verbose: bool,
}

impl Configuration {
    pub fn new(search_path: &Path, idl_file: &Path, verbose: bool) -> Self {
        Self {
            search_path: search_path.to_path_buf(),
            idl_file: idl_file.to_path_buf(),
            verbose,
        }
    }
}

/// Vec to modules. Lower indexes are 'owners' of higher indexes.
type Scope = Vec<String>;

/// Container for the configuration & root module
#[derive(Debug, Clone)]
struct Context<'i> {
    config: &'i Configuration,
    root_module: IdlModule,
}

impl<'i> Context<'i> {
    pub fn new(config: &'i Configuration) -> Context<'i> {
        Context {
            config,
            root_module: IdlModule::new(None),
        }
    }

    /// Find the desired module under the root_module or create it if it
    /// does not already exist. The module request is made via the scope.
    fn lookup_module(&mut self, scope: &Scope) -> &mut IdlModule {
        // Starting from Root traverse the scope-path
        let mut current_module = &mut self.root_module;

        for name in scope {
            let submodule = current_module
                .modules
                .entry(name.to_owned())
                .or_insert(IdlModule::new(Some(name.to_owned())));
            current_module = submodule;
        }

        current_module
    }

    /// Add a new entry to the module for the discovered type
    fn add_type_dcl(&mut self, scope: &Scope, key: String, type_dcl: IdlTypeDcl) {
        self.lookup_module(scope)
            .types
            .entry(key)
            .or_insert(type_dcl);
    }

    /// Add a new entry to the module for the discovered const
    fn add_const_dcl(&mut self, scope: &Scope, key: String, const_dcl: IdlConstDcl) {
        self.lookup_module(scope)
            .constants
            .entry(key)
            .or_insert(const_dcl);
    }

    /// type_spec = { template_type_spec | simple_type_spec }
    pub fn read_type_spec(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlTypeSpec, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        let type_spec = match pair.as_rule() {
            Rule::float => IdlTypeSpec::F32Type,
            Rule::double => IdlTypeSpec::F64Type,
            Rule::long_double => IdlTypeSpec::F128Type,
            Rule::unsigned_short_int => IdlTypeSpec::U16Type,
            Rule::unsigned_longlong_int => IdlTypeSpec::U64Type,
            Rule::unsigned_long_int => IdlTypeSpec::U32Type,
            Rule::signed_short_int => IdlTypeSpec::I16Type,
            Rule::signed_longlong_int => IdlTypeSpec::I64Type,
            Rule::signed_long_int => IdlTypeSpec::I32Type,
            Rule::char_type => IdlTypeSpec::CharType,
            Rule::wide_char_type => IdlTypeSpec::WideCharType,
            Rule::boolean_type => IdlTypeSpec::BooleanType,
            Rule::octet_type => IdlTypeSpec::OctetType,
            Rule::string_type => match iter.next() {
                None => IdlTypeSpec::StringType(None),
                Some(ref p) => {
                    let pos_int_const = self.read_const_expr(scope, p)?;
                    IdlTypeSpec::StringType(Some(Box::new(pos_int_const)))
                }
            },
            Rule::wide_string_type => match iter.next() {
                None => IdlTypeSpec::WideStringType(None),
                Some(ref p) => {
                    let pos_int_const = self.read_const_expr(scope, p)?;
                    IdlTypeSpec::WideStringType(Some(Box::new(pos_int_const))) // Needs to be a &str
                }
            },
            Rule::sequence_type => match (iter.next(), iter.next()) {
                (Some(ref typ), None) => {
                    let typ_expr = self.read_type_spec(scope, typ)?;
                    IdlTypeSpec::SequenceType(Box::new(typ_expr))
                }
                (Some(ref typ), Some(ref bound)) => {
                    let typ_expr = self.read_type_spec(scope, typ)?;
                    let _bound_expr = self.read_const_expr(scope, bound)?;
                    IdlTypeSpec::SequenceType(Box::new(typ_expr))
                }
                _ => panic!(),
            },
            //  scoped_name = { "::"? ~ identifier ~ ("::" ~ identifier)* }
            Rule::scoped_name => {
                let name = self.read_scoped_name(scope, pair)?;
                IdlTypeSpec::ScopedName(name)
            }
            // go deeper
            _ => {
                let p = pair.clone().into_inner().next().unwrap();
                self.read_type_spec(scope, &p)?
            }
        };

        Ok(type_spec)
    }

    /// declarator = { array_declarator | simple_declarator }
    /// array_declarator = { identifier ~ fixed_array_size+ }
    /// simple_declarator = { identifier }
    pub fn read_struct_member_declarator(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
        type_spec: &IdlTypeSpec,
    ) -> Result<IdlStructMember, IdlError> {
        let decl = pair.clone().into_inner().next().unwrap();

        let mut iter = decl.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("should be declarator {:?}", decl.as_rule());
        }
        match decl.as_rule() {
            // simple_declarator = { identifier }
            Rule::simple_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let member_dcl = IdlStructMember {
                    id,
                    type_spec: type_spec.clone(),
                };

                Ok(member_dcl)
            }

            // array_declarator = { identifier ~ fixed_array_size+ }
            Rule::array_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let array_sizes: Result<Vec<_>, IdlError> = iter
                    .map(|p|
                            // skip node Rule::fixed_array_size and read const_expr underneath
                            self.read_const_expr(
                                scope,
                                &p.clone().into_inner().next().unwrap()))
                    .collect();
                let array_type_spec =
                    IdlTypeSpec::ArrayType(Box::new(type_spec.clone()), array_sizes?);

                let member_dcl = IdlStructMember {
                    id,
                    type_spec: array_type_spec,
                };

                Ok(member_dcl)
            }

            _ => Err(IdlError::InternalError),
        }
    }

    // member = { type_spec ~ declarators ~ ";" }
    // declarators = { declarator ~ ("," ~ declarator )* }
    // declarator = { array_declarator | simple_declarator }
    fn read_struct_member(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<Vec<IdlStructMember>, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        let type_spec = self.read_type_spec(scope, &iter.next().unwrap())?;

        // skip rule 'declarators' and parse sibblings `declarator'
        let declarators = iter.next().unwrap().clone().into_inner();

        declarators
            .map(|declarator| self.read_struct_member_declarator(scope, &declarator, &type_spec))
            .collect()
    }

    /// identifier = @{ (alpha | "_") ~ ("_" | alpha | digit)* }
    fn read_identifier(&mut self, scope: &Scope, pair: &Pair<Rule>) -> Result<String, IdlError> {
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        match pair.as_rule() {
            Rule::identifier | Rule::enumerator => Ok(pair.as_str().to_owned()),
            _ => Err(IdlError::ExpectedItem(Rule::identifier)),
        }
    }

    /// scoped_name = { "::"? ~ identifier ~ ("::" ~ identifier)* }
    fn read_scoped_name(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlScopedName, IdlError> {
        let iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!(">>> {:?} '{}'", pair.as_rule(), pair.as_str());
        }
        // check if name starts with "::"
        let is_absolute_name = pair.as_str().starts_with("::");
        let scoped_name = iter
            .map(|p| self.read_identifier(scope, &p).unwrap().to_owned())
            .collect::<Vec<String>>();

        Ok(IdlScopedName(scoped_name, is_absolute_name))
    }

    /// const_expr = { unary_expr ~ (or_expr | xor_expr | and_expr | shift_expr | add_expr | mult_expr)? }
    fn read_const_expr(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlValueExpr, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?} '{}'", pair.as_rule(), pair.as_str());
        }
        let fp_collect_init = (None, None, None, None);

        let fp_collect = |(i, f, e, s), node: Pair<Rule>| match node.as_rule() {
            Rule::integral_part => (Some(node.as_str().to_owned()), f, e, s),
            Rule::fractional_part => (i, Some(node.as_str().to_owned()), e, s),
            Rule::exponent => (i, f, Some(node.as_str().to_owned()), s),
            Rule::float_suffix => (i, f, e, Some(node.as_str().to_owned())),
            _ => panic!(),
        };

        match pair.as_rule() {
            Rule::const_expr => match (iter.next(), iter.next()) {
                (Some(ref expr1), Some(ref expr2)) => {
                    let e1 = self.read_const_expr(scope, expr1)?;
                    let e2 = self.read_const_expr(scope, expr2)?;
                    Ok(IdlValueExpr::Expr(Box::new(e1), Box::new(e2)))
                }
                (Some(ref expr1), None) => self.read_const_expr(scope, expr1),
                _ => Err(IdlError::ExpectedItem(Rule::const_expr)),
            },
            Rule::unary_expr => match (iter.next(), iter.next()) {
                (Some(ref unary_op), Some(ref prim_expr)) => {
                    // TBD
                    let expr = self.read_const_expr(scope, prim_expr)?;
                    match unary_op.as_str() {
                        "-" => Ok(IdlValueExpr::UnaryOp(UnaryOp::Neg, Box::new(expr))),
                        "+" => Ok(IdlValueExpr::UnaryOp(UnaryOp::Pos, Box::new(expr))),
                        "~" => Ok(IdlValueExpr::UnaryOp(UnaryOp::Inverse, Box::new(expr))),
                        _ => Err(IdlError::ExpectedItem(Rule::unary_operator)),
                    }
                }
                (Some(ref prim_expr), None) => self.read_const_expr(scope, prim_expr),
                _ => Err(IdlError::ExpectedItem(Rule::primary_expr)),
            },
            Rule::primary_expr => match iter.next() {
                //  scoped_name = { "::"? ~ identifier ~ ("::" ~ identifier)* }
                Some(ref p) if p.as_rule() == Rule::scoped_name => {
                    let name = self.read_scoped_name(scope, p)?;
                    Ok(IdlValueExpr::ScopedName(name))
                }
                Some(ref p) if p.as_rule() == Rule::literal => self.read_const_expr(scope, p),
                Some(ref p) if p.as_rule() == Rule::const_expr => {
                    let expr = self.read_const_expr(scope, p)?;
                    Ok(IdlValueExpr::Brace(Box::new(expr)))
                }
                _ => Err(IdlError::ExpectedItem(Rule::primary_expr)),
            },
            Rule::and_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::And, Box::new(expr)))
            }
            Rule::or_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Or, Box::new(expr)))
            }
            Rule::xor_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Xor, Box::new(expr)))
            }
            Rule::lshift_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::LShift, Box::new(expr)))
            }
            Rule::rshift_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::RShift, Box::new(expr)))
            }
            Rule::add_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Add, Box::new(expr)))
            }
            Rule::sub_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Sub, Box::new(expr)))
            }
            Rule::mul_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Mul, Box::new(expr)))
            }
            Rule::div_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Div, Box::new(expr)))
            }
            Rule::mod_expr => {
                let expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                Ok(IdlValueExpr::BinaryOp(BinaryOp::Mod, Box::new(expr)))
            }
            Rule::decimal_integer_literal => Ok(IdlValueExpr::DecLiteral(pair.as_str().to_owned())),
            Rule::octal_integer_literal => Ok(IdlValueExpr::OctLiteral(pair.as_str().to_owned())),
            Rule::hex_integer_literal => Ok(IdlValueExpr::HexLiteral(pair.as_str().to_owned())),
            Rule::floating_pt_literal => {
                let (i, f, e, s) = iter.fold(fp_collect_init, fp_collect);
                Ok(IdlValueExpr::FloatLiteral(i, f, e, s))
            }
            Rule::boolean_literal => match pair.as_str() {
                "TRUE" => Ok(IdlValueExpr::BooleanLiteral(true)),
                _ => Ok(IdlValueExpr::BooleanLiteral(false)),
            },
            Rule::character_literal => Ok(IdlValueExpr::CharLiteral(pair.as_str().to_owned())),
            Rule::wide_character_literal => {
                Ok(IdlValueExpr::WideCharLiteral(pair.as_str().to_owned()))
            }
            Rule::string_literal => Ok(IdlValueExpr::StringLiteral(pair.as_str().to_owned())),
            Rule::wide_string_literal => {
                Ok(IdlValueExpr::WideStringLiteral(pair.as_str().to_owned()))
            }
            _ => self.read_const_expr(scope, &iter.next().unwrap()),
        }
    }

    /// declarator = { array_declarator | simple_declarator }
    /// array_declarator = { identifier ~ fixed_array_size+ }
    /// simple_declarator = { identifier }
    fn process_declarator(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
        type_spec: &IdlTypeSpec,
    ) -> Result<(), IdlError> {
        let decl = pair.clone().into_inner().next().unwrap();
        let mut iter = decl.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", decl.as_rule());
        }
        match decl.as_rule() {
            // simple_declarator = { identifier }
            Rule::simple_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;

                let type_dcl = IdlTypeDcl(IdlTypeDclKind::TypeDcl(id.clone(), type_spec.clone()));
                self.add_type_dcl(scope, id, type_dcl);
                Ok(())
            }

            // array_declarator = { identifier ~ fixed_array_size+ }
            Rule::array_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let key = id.clone();

                let array_sizes: Result<Vec<_>, IdlError> = iter
                    .map(|p|
                            // skip node Rule::fixed_array_size and read const_expr underneath
                            self.read_const_expr(
                                scope,
                                &p.clone().into_inner().next().unwrap()))
                    .collect();
                let array_type_spec =
                    IdlTypeSpec::ArrayType(Box::new(type_spec.clone()), array_sizes?);
                let type_dcl = IdlTypeDcl(IdlTypeDclKind::TypeDcl(id, array_type_spec));
                self.add_type_dcl(scope, key, type_dcl);
                Ok(())
            }
            // traverse deeper
            _ => Err(IdlError::InternalError),
        }
    }

    /// Walk through all discovered pairs and create the associated objs
    fn process<L: IdlLoader>(
        &mut self,
        scope: &mut Scope,
        loader: &mut dyn IdlLoader,
        pair: &Pair<Rule>,
    ) -> Result<(), IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        match pair.as_rule() {
            // module_dcl = { "module" ~ identifier ~ "{" ~ definition* ~ "}" }
            Rule::module_dcl => {
                let id = iter.next().unwrap().as_str();

                scope.push(id.to_owned());

                let _ = self.lookup_module(scope);

                for p in iter {
                    let _ = self.process::<L>(scope, loader, &p);
                }

                let _ = scope.pop();

                Ok(())
            }
            // struct_def = { "struct" ~ identifier ~ (":" ~ scoped_name)? ~ "{" ~ member* ~ "}" }
            Rule::struct_def => {
                let id = iter.next().unwrap().as_str().to_owned();
                let key = id.clone();
                let m1: Result<Vec<Vec<IdlStructMember>>, _> = iter
                    .map(|p| {
                        // skip the member-node and read sibbling directly
                        self.read_struct_member(scope, &p)
                    })
                    .collect();

                let m2 = m1?;
                let members = m2.into_iter().flatten().collect::<Vec<_>>();

                let typedcl = IdlTypeDcl(IdlTypeDclKind::StructDcl(id, members));
                self.add_type_dcl(scope, key, typedcl);
                Ok(())
            }
            // union_def = { "union" ~ identifier ~ "switch" ~ "(" ~ switch_type_spec ~ ")" ~ "{" ~ switch_body ~ "}" }
            Rule::union_def => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let key = id.to_owned();
                let switch_type_spec = self.read_switch_type_spec(scope, &iter.next().unwrap())?;
                let switch_body = self.read_switch_body(scope, &iter.next().unwrap())?;
                let union_def =
                    IdlTypeDcl(IdlTypeDclKind::UnionDcl(id, switch_type_spec, switch_body));

                self.add_type_dcl(scope, key, union_def);
                Ok(())
            }
            // type_declarator = { (template_type_spec | constr_type_dcl | simple_type_spec) ~ any_declarators }
            Rule::type_declarator => {
                let type_spec = self.read_type_spec(scope, &iter.next().unwrap())?;

                let any_declarators_pair = &iter.next().unwrap();

                for p in any_declarators_pair.clone().into_inner() {
                    let _ = self.process_declarator(scope, &p, &type_spec);
                }
                Ok(())
            }
            // enum_dcl = { "enum" ~ identifier ~ "{" ~ enumerator ~ ("," ~ enumerator)* ~ ","? ~ "}" }
            // enumerator = { identifier }
            Rule::enum_dcl => {
                let id = iter.next().unwrap().as_str().to_owned();
                let key = id.clone();
                let enums: Result<Vec<_>, IdlError> =
                    iter.map(|p| self.read_identifier(scope, &p)).collect();

                let typedcl = IdlTypeDcl(IdlTypeDclKind::EnumDcl(id, enums?));
                self.add_type_dcl(scope, key, typedcl);
                Ok(())
            }
            // const_dcl = { "const" ~ const_type ~ identifier ~ "=" ~ const_expr }
            Rule::const_dcl => {
                let type_spec = self.read_type_spec(scope, &iter.next().unwrap())?;
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let key = id.clone();
                let const_expr = self.read_const_expr(scope, &iter.next().unwrap())?;
                let const_dcl = IdlConstDcl {
                    id,
                    typedcl: type_spec,
                    value: const_expr,
                };
                self.add_const_dcl(scope, key, const_dcl);
                Ok(())
            }
            // include_directive = !{ "#" ~ "include" ~ (("<" ~ path_spec ~ ">") | ("\"" ~ path_spec ~ "\"")) }
            Rule::include_directive => {
                if let Some(ref p) = pair.clone().into_inner().nth(0) {
                    let fname = p.as_str();
                    let data = loader
                        .load(&PathBuf::from(fname))
                        .map_err(|_| IdlError::FileNotFound(fname.to_owned()))?;

                    let idl: Pairs<Rule> = IdlParser::parse(Rule::specification, &data)
                        .map_err(|e| IdlError::ErrorMesg(e.to_string()))?;

                    for p in idl {
                        self.process::<L>(scope, loader, &p)?;
                    }
                }
                Ok(())
            }
            // anything else
            _ => {
                for p in iter {
                    let _ = self.process::<L>(scope, loader, &p);
                }
                Ok(())
            }
        }
    }

    /// declarator = { array_declarator | simple_declarator }
    /// array_declarator = { identifier ~ fixed_array_size+ }
    /// simple_declarator = { identifier }
    fn read_switch_element_declarator(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
        type_spec: &IdlTypeSpec,
    ) -> Result<IdlSwitchElement, IdlError> {
        let decl = pair.clone().into_inner().next().unwrap();

        let mut iter = decl.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("should be declarator {:?}", decl.as_rule());
        }
        match decl.as_rule() {
            // simple_declarator = { identifier }
            Rule::simple_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let member_dcl = IdlSwitchElement {
                    id,
                    type_spec: type_spec.clone(),
                };

                Ok(member_dcl)
            }

            // array_declarator = { identifier ~ fixed_array_size+ }
            Rule::array_declarator => {
                let id = self.read_identifier(scope, &iter.next().unwrap())?;
                let array_sizes: Result<Vec<_>, IdlError> = iter
                    .map(|p|
                            // skip node Rule::fixed_array_size and read const_expr underneath
                            self.read_const_expr(
                                scope,
                                &p.clone().into_inner().next().unwrap()))
                    .collect();
                let array_type_spec =
                    IdlTypeSpec::ArrayType(Box::new(type_spec.clone()), array_sizes?);

                let member_dcl = IdlSwitchElement {
                    id,
                    type_spec: array_type_spec,
                };

                Ok(member_dcl)
            }

            _ => Err(IdlError::InternalError),
        }
    }

    /// element_spec = { type_spec ~ declarator }
    fn read_switch_element_spec(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlSwitchElement, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        let type_spec = self.read_type_spec(scope, &iter.next().unwrap())?;
        self.read_switch_element_declarator(scope, &iter.next().unwrap(), &type_spec)
    }

    /// switch_type_spec = {integer_type | char_type | boolean_type | wide_char_type| octet_type| scoped_name }
    fn read_switch_type_spec(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlTypeSpec, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }
        self.read_type_spec(scope, &iter.next().unwrap())
    }

    /// switch_body = { case+ }
    fn read_switch_body(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<Vec<IdlSwitchCase>, IdlError> {
        let iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }

        let cases: Result<Vec<_>, IdlError> =
            iter.map(|p| self.read_switch_case(scope, &p)).collect();

        cases
    }

    /// case_label = { "case" ~ const_expr ~ ":" | "default" ~ ":" }
    fn read_switch_label(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlSwitchLabel, IdlError> {
        let mut iter = pair.clone().into_inner();
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }

        match iter.next() {
            Some(p) => {
                let expr = self.read_const_expr(scope, &p)?;
                Ok(IdlSwitchLabel::Label(expr))
            }
            _ => Ok(IdlSwitchLabel::Default),
        }
    }

    /// case = { case_label+ ~ element_spec ~ ";" }
    fn read_switch_case(
        &mut self,
        scope: &Scope,
        pair: &Pair<Rule>,
    ) -> Result<IdlSwitchCase, IdlError> {
        if self.config.verbose {
            print!("{:indent$}", "", indent = 3 * scope.len());
            println!("{:?}", pair.as_rule());
        }

        let case_labels: Result<Vec<IdlSwitchLabel>, IdlError> = pair
            .clone()
            .into_inner()
            .filter(|p| p.as_rule() == Rule::case_label)
            .map(|p| self.read_switch_label(scope, &p))
            .collect();

        // there will be only one in the list, choose the last
        let elem_spec = pair
            .clone()
            .into_inner()
            .filter(|p| p.as_rule() == Rule::element_spec)
            .map(|p| self.read_switch_element_spec(scope, &p))
            .last()
            .unwrap();

        Ok(IdlSwitchCase {
            labels: case_labels?,
            elem_spec: elem_spec?,
        })
    }

    // enum_dcl = { "enum" ~ identifier ~ "{" ~ enumerator ~ ("," ~ enumerator)* ~ ","? ~ "}" }
    // enumerator = { identifier }
}

/// Provided w/ an object that supports writing, an IDL Loader, and an OMG Gen Config,
/// generate Rust Types for the requested OMG IDL files.
///
/// @param out: An object that supports writing
/// @param loader: Library Object to read IDL
/// @param config: Library config
fn generate_with_loader<W: Write, L: IdlLoader>(
    out: &mut W,
    loader: &mut L,
    config: &Configuration,
) -> Result<(), IdlError> {
    let mut ctx = Context::new(config);

    let idl_file_data = loader
        .load(&config.idl_file)
        .map_err(|_| IdlError::FileNotFound("Could not find requested idl_file".to_string()))?;

    let idl: Pairs<Rule> = IdlParser::parse(Rule::specification, &idl_file_data)
        .map_err(|e| IdlError::ErrorMesg(e.to_string()))?;

    let mut scope = Scope::new();

    for p in idl {
        let _ = ctx.process::<L>(&mut scope, loader, &p);
    }

    let mut env = minijinja::Environment::new();
    minijinja_embed::load_templates!(&mut env);
    let root_module_text = ctx
        .root_module
        .render(&env, 0)
        .map_err(|_| IdlError::InternalError)?;

    write!(out, "{root_module_text}").map_err(|_| IdlError::InternalError)
}

/// Object used to input the request IDL file into the library.
#[derive(Debug, Clone, Default)]
struct Loader {
    search_path: PathBuf,
}

impl Loader {
    pub fn new(search_path: &Path) -> Self {
        Self {
            search_path: search_path.to_path_buf(),
        }
    }
}

impl IdlLoader for Loader {
    /// Read the requested file and return as a Result<String>
    fn load(&self, filename: &Path) -> Result<String, Error> {
        let fullname = self.search_path.join(filename);
        let mut file = File::open(fullname)?;
        let mut data = String::new();

        file.read_to_string(&mut data)?;

        Ok(data)
    }
}

/// Provided w/ an object that supports writing and a OMG Gen Config
/// generate Rust Types for the requested OMG IDL files.
///
/// @param out: An object that supports writing
/// @param config: Library config
pub fn generate_with_search_path<W: Write>(
    out: &mut W,
    config: &Configuration,
) -> Result<(), IdlError> {
    let mut loader = Loader::new(&config.search_path);
    let mut env = minijinja::Environment::new();
    minijinja_embed::load_templates!(&mut env);
    generate_with_loader(out, &mut loader, config)
}
