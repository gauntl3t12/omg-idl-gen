// Copyright (C) 2019  Frank Rehberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>
use linked_hash_map::LinkedHashMap;
use serde_derive::Serialize;
use std::fmt;

const INDENTION: usize = 4;
const IMPORT_VEC: &str = "use std::vec::Vec;";
const ATTR_ALLOW_UNUSED_IMPORTS: &str = "#[allow(unused_imports)]";
const IMPORT_SERDE: &str = "use serde_derive::{Serialize, Deserialize};";

/// Enum representing a basic operator that applies to a single value.
/// i.e. + (Positive), - (Negative), ~ (Inverse)
#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Pos,
    Inverse,
}

impl UnaryOp {
    /// Convert the UnaryOp enumeration into a &str
    pub fn to_str(&self) -> &str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Pos => "+",
            UnaryOp::Inverse => "~",
        }
    }
}

/// Enum representing an operator on two values. I.e. + (Add), - (Sub),
/// * (Mul), / (Div), % (Mod), < (LShift), > (RShift), | (Or), ^ (Xor),
///   & (And)
#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LShift,
    RShift,
    Or,
    Xor,
    And,
}

impl BinaryOp {
    /// Convert the BinaryOp enumeration into a &str
    pub fn to_str(&self) -> &str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::LShift => "<<",
            BinaryOp::RShift => ">>",
            BinaryOp::Or => "|",
            BinaryOp::Xor => "^",
            BinaryOp::And => "&",
        }
    }
}

/// A name under scope, I.e. crate::cmn
#[derive(Clone, Debug)]
pub struct IdlScopedName(pub Vec<String>, pub bool);

impl fmt::Display for IdlScopedName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let is_absolute_path = self.1;
        let components = &self.0;
        for (idx, comp) in components.iter().enumerate() {
            // TODO, use paths according to "crate::" or "super::"
            if idx == 0 && !is_absolute_path {
                write!(f, "{comp}")?
            } else if idx == 0 && is_absolute_path {
                write!(f, "crate::{comp}")?
            } else {
                write!(f, "::{comp}")?
            }
        }
        Ok(())
    }
}

/// Representation of all the different right hand options in an equation.
#[derive(Clone, Debug, Default)]
pub enum IdlValueExpr {
    #[default]
    None,
    DecLiteral(String),
    HexLiteral(String),
    OctLiteral(String),
    CharLiteral(String),
    WideCharLiteral(String),
    StringLiteral(String),
    WideStringLiteral(String),
    BooleanLiteral(bool),
    FloatLiteral(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    UnaryOp(UnaryOp, Box<IdlValueExpr>),
    BinaryOp(BinaryOp, Box<IdlValueExpr>),
    Expr(Box<IdlValueExpr>, Box<IdlValueExpr>),
    Brace(Box<IdlValueExpr>),
    ScopedName(IdlScopedName),
}

impl fmt::Display for IdlValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value_expr = match self {
            IdlValueExpr::None => "",
            IdlValueExpr::DecLiteral(val) => val,
            IdlValueExpr::HexLiteral(val) => val,
            IdlValueExpr::OctLiteral(val) => val,
            IdlValueExpr::CharLiteral(val) => val,
            IdlValueExpr::WideCharLiteral(val) => val,
            IdlValueExpr::StringLiteral(val) => val,
            IdlValueExpr::WideStringLiteral(val) => val,
            IdlValueExpr::BooleanLiteral(val) => &val.to_string(),
            IdlValueExpr::UnaryOp(op, expr) => &format!("{}{}", op.to_str(), expr),
            IdlValueExpr::BinaryOp(op, expr) => &format!("{}{}", op.to_str(), expr),
            IdlValueExpr::Expr(expr1, expr2) => &format!("{}{}", expr1, expr2),
            IdlValueExpr::Brace(expr) => &format!("({})", expr),
            IdlValueExpr::FloatLiteral(integral, fraction, exponent, suffix) => &format!(
                "{}.{}e{}{}",
                integral.as_ref().unwrap().clone(),
                fraction.as_ref().unwrap().clone(),
                exponent.as_ref().unwrap().clone(),
                suffix.as_ref().unwrap().clone()
            ),
            IdlValueExpr::ScopedName(name) => &name.to_string(),
        };
        write!(f, "{value_expr}")
    }
}

/// Representation of an IDL Struct
#[derive(Clone, Debug)]
pub struct IdlStructMember {
    pub id: String,
    pub type_spec: IdlTypeSpec,
}

/// Representation of an IDL Switch
#[derive(Clone, Debug)]
pub struct IdlSwitchElement {
    pub id: String,
    pub type_spec: IdlTypeSpec,
}

/// Representation of an IDL Switch Label
#[derive(Clone, Debug)]
pub enum IdlSwitchLabel {
    Label(IdlValueExpr),
    Default,
}

/// Representation of an IDL Switch Case
#[derive(Clone, Debug)]
pub struct IdlSwitchCase {
    pub labels: Vec<IdlSwitchLabel>,
    pub elem_spec: IdlSwitchElement,
}

/// Representation of an IDL Type
#[derive(Clone, Debug, Default)]
pub enum IdlTypeSpec {
    #[default]
    None,
    ArrayType(Box<IdlTypeSpec>, Vec<IdlValueExpr>),
    SequenceType(Box<IdlTypeSpec>),
    StringType(Option<Box<IdlValueExpr>>),
    WideStringType(Option<Box<IdlValueExpr>>),
    // FixedPtType,
    // EnumDcl,
    // BitsetDcl,
    // BitmaskDcl,
    F32Type,
    F64Type,
    F128Type,
    I16Type,
    I32Type,
    I64Type,
    U16Type,
    U32Type,
    U64Type,
    CharType,
    WideCharType,
    BooleanType,
    OctetType,
    // AnyType,
    // ObjectType,
    // ValueBaseType,
    ScopedName(IdlScopedName),
}

impl fmt::Display for IdlTypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value_expr = match self {
            IdlTypeSpec::F32Type => "f32",
            IdlTypeSpec::F64Type => "f64",
            IdlTypeSpec::F128Type => "f128",
            IdlTypeSpec::I16Type => "i16",
            IdlTypeSpec::I32Type => "i32",
            IdlTypeSpec::I64Type => "i64",
            IdlTypeSpec::U16Type => "u16",
            IdlTypeSpec::U32Type => "u32",
            IdlTypeSpec::U64Type => "u64",
            IdlTypeSpec::CharType => "char",
            IdlTypeSpec::WideCharType => "char",
            IdlTypeSpec::BooleanType => "bool",
            IdlTypeSpec::OctetType => "u8",
            IdlTypeSpec::StringType(None) => "String",
            IdlTypeSpec::WideStringType(None) => "String",
            // TODO implement String/Sequence bounds
            IdlTypeSpec::StringType(_) => "String",
            // TODO implement String/Sequence bounds for serializer and deserialzer
            IdlTypeSpec::WideStringType(_) => "String",
            IdlTypeSpec::SequenceType(typ_expr) => &format!("Vec<{}>", typ_expr.as_ref()),
            IdlTypeSpec::ArrayType(typ_expr, dim_expr_list) => {
                let dim_list_str = dim_expr_list
                    .iter()
                    .map(|expr| format!(";{}]", expr))
                    .collect::<String>();
                &format!(
                    "{}{}{dim_list_str}",
                    "[".repeat(dim_expr_list.len()),
                    typ_expr
                )
            }
            IdlTypeSpec::ScopedName(name) => &name.to_string(),
            _ => unimplemented!(),
        };
        write!(f, "{value_expr}")
    }
}

/// Selector for the different supported IDL kinds
#[derive(Clone, Debug, Default)]
pub enum IdlTypeDclKind {
    #[default]
    None,
    TypeDcl(String, IdlTypeSpec),
    StructDcl(String, Vec<IdlStructMember>),
    UnionDcl(String, IdlTypeSpec, Vec<IdlSwitchCase>),
    EnumDcl(String, Vec<String>),
}

/// Representation of an IDL Type
#[derive(Clone, Debug, Default)]
pub struct IdlTypeDcl(pub IdlTypeDclKind);

/// Data storage to align with Jinja (IdlStruct)
#[derive(Serialize)]
struct IdlStructField {
    name: String,
    type_str: String,
}

/// Data storage to align with Jinja (IdlSwitch)
#[derive(Serialize)]
struct IdlSwitchField {
    name: String,
    element_id: String,
    element_type: String,
}

impl IdlTypeDcl {
    /// Convert the object to a Result<String> for output. The env must have the templates
    /// already loaded.
    pub fn render(
        &mut self,
        env: &minijinja::Environment,
        level: usize,
    ) -> Result<String, minijinja::Error> {
        match self.0 {
            IdlTypeDclKind::TypeDcl(ref id, ref type_spec) => {
                let tmpl = env.get_template("typedef.j2")?;
                tmpl.render(minijinja::context! {
                    typedef_name => id,
                    typedef_type => type_spec.to_string(),
                    indent_level => level
                })
            }
            IdlTypeDclKind::StructDcl(ref id, ref type_spec) => {
                let tmpl = env.get_template("struct.j2")?;
                let fields = type_spec
                    .iter()
                    .map(|field| IdlStructField {
                        name: field.id.clone(),
                        type_str: field.type_spec.to_string(),
                    })
                    .collect::<Vec<IdlStructField>>();

                tmpl.render(minijinja::context! {
                    struct_name => id,
                    fields,
                    indent_level => level
                })
            }
            IdlTypeDclKind::EnumDcl(ref id, ref enums) => {
                let tmpl = env.get_template("enum.j2")?;
                tmpl.render(minijinja::context! {
                    enum_name => id,
                    variants => enums,
                    indent_level => level
                })
            }
            IdlTypeDclKind::UnionDcl(ref id, ref _type_spec, ref switch_cases) => {
                let tmpl = env.get_template("union_switch.j2")?;
                let union_members = switch_cases
                    .iter()
                    .flat_map(|case| {
                        case.labels
                            .clone()
                            .iter()
                            .map(|label| {
                                let label = match label {
                                    IdlSwitchLabel::Label(label) => label.to_string(),
                                    IdlSwitchLabel::Default => "default".to_owned(),
                                };

                                IdlSwitchField {
                                    name: label.to_string(),
                                    element_id: case.elem_spec.id.clone(),
                                    element_type: case.elem_spec.type_spec.to_string(),
                                }
                            })
                            .collect::<Vec<IdlSwitchField>>()
                    })
                    .collect::<Vec<IdlSwitchField>>();

                tmpl.render(minijinja::context! {
                    union_name => id,
                    union_members,
                    indent_level => level
                })
            }
            IdlTypeDclKind::None => Ok(String::new()),
        }
    }
}

/// Data representation of a const declaration
#[derive(Clone, Default, Debug)]
pub struct IdlConstDcl {
    pub id: String,
    pub typedcl: IdlTypeSpec,
    pub value: IdlValueExpr,
}

impl IdlConstDcl {
    /// Convert the object to a Result<String> for output. The env must have the templates
    /// already loaded.
    pub fn render(
        &mut self,
        env: &minijinja::Environment,
        level: usize,
    ) -> Result<String, minijinja::Error> {
        let tmpl = env.get_template("const.j2")?;
        tmpl.render(minijinja::context! {
            const_name => self.id,
            const_type => self.typedcl.to_string(),
            const_value => self.value.to_string(),
            indent_level => level
        })
    }
}

/// Data representation of an IDL Module or the root module.
#[derive(Clone, Default, Debug)]
pub struct IdlModule {
    pub id: Option<String>,
    pub uses: Vec<String>,
    pub modules: LinkedHashMap<String, IdlModule>,
    pub types: LinkedHashMap<String, IdlTypeDcl>,
    pub constants: LinkedHashMap<String, IdlConstDcl>,
}

impl IdlModule {
    pub fn new(id: Option<String>) -> IdlModule {
        IdlModule {
            id,
            uses: Vec::new(),
            modules: LinkedHashMap::default(),
            types: LinkedHashMap::default(),
            constants: LinkedHashMap::default(),
        }
    }

    /// Convert the object to a Result<String> for output. The env must have the templates
    /// already loaded.
    pub fn render(
        &mut self,
        env: &minijinja::Environment,
        level: usize,
    ) -> Result<String, minijinja::Error> {
        let mut module_info = String::new();
        let add = if self.id.is_some() { 1 } else { 0 };

        // TODO populate this instead of hardcoded solution
        let indent = level * INDENTION;
        for required_use in &self.uses {
            let uses = format!(
                "{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{required_use}\n",
                "", ""
            );
            module_info.push_str(&uses);
        }

        let use_vec = format!(
            "{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{IMPORT_VEC}\n",
            "",
            "",
            indent = (level + add) * INDENTION
        );
        let use_serde = format!(
            "{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{IMPORT_SERDE}\n",
            "",
            "",
            indent = (level + add) * INDENTION
        );
        module_info.push_str(&use_vec);
        module_info.push_str(&use_serde);

        for typ in self.types.entries() {
            let rendered = typ.into_mut().render(env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        for module in self.modules.entries() {
            let rendered = module.into_mut().render(env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        for cnst in self.constants.entries() {
            let rendered = cnst.into_mut().render(env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        match self.id {
            Some(ref id_str) => {
                let tmpl = env.get_template("module.j2")?;
                tmpl.render(minijinja::context! {
                    module_name => id_str,
                    module_information => module_info,
                    indent_level => level
                })
            }
            None => Ok(module_info),
        }
    }
}
