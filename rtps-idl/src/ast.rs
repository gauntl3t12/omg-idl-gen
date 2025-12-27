// Copyright (C) 2019  Frank Rehberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>
use minijinja;
use linked_hash_map::LinkedHashMap;
use serde_derive::Serialize;

const INDENTION: usize = 4;
const IMPORT_VEC: &str = "use std::vec::Vec;";
const ATTR_ALLOW_UNUSED_IMPORTS: &str = "#[allow(unused_imports)]";
const IMPORT_SERDE: &str = "use serde_derive::{Serialize, Deserialize};";

///
#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Pos,
    Inverse,
}

impl UnaryOp {
    pub fn to_str(&self) -> &str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Pos => "+",
            UnaryOp::Inverse => "~",
        }
    }
}

///
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

///
#[derive(Clone, Debug)]
pub struct IdlScopedName(pub Vec<String>, pub bool);

impl IdlScopedName {
    pub fn to_string(&self) -> String {
        let is_absolute_path = self.1;
        let components = &self.0;
        let mut ret = String::new();
        for (idx, comp) in components.iter().enumerate() {
            // TODO, use paths according to "crate::" or "super::"
            if idx == 0 && !is_absolute_path {
                ret.push_str(comp);
            } else if idx == 0 && is_absolute_path {
                let crate_comp = format!("crate::{}", comp);
                ret.push_str(&crate_comp);
            } else {
                let layer_comp = format!("::{}", comp);
                ret.push_str(&layer_comp);
            }
        }
        ret
    }
}

///
#[derive(Clone, Debug)]
pub enum IdlValueExpr {
    None,
    DecLiteral(String),
    HexLiteral(String),
    OctLiteral(String),
    CharLiteral(String),
    WideCharLiteral(String),
    StringLiteral(String),
    WideStringLiteral(String),
    BooleanLiteral(bool),
    FloatLiteral(Option<String>, Option<String>, Option<String>, Option<String>),
    UnaryOp(UnaryOp, Box<IdlValueExpr>),
    BinaryOp(BinaryOp, Box<IdlValueExpr>),
    Expr(Box<IdlValueExpr>, Box<IdlValueExpr>),
    Brace(Box<IdlValueExpr>),
    ScopedName(IdlScopedName),
}

impl IdlValueExpr {
    pub fn to_string(&self) -> String {
        match self {
            IdlValueExpr::None => "".to_string(),
            IdlValueExpr::DecLiteral(val) => val.to_owned(),
            IdlValueExpr::HexLiteral(val) => val.to_owned(),
            IdlValueExpr::OctLiteral(val) => val.to_owned(),
            IdlValueExpr::CharLiteral(val) => val.to_owned(),
            IdlValueExpr::WideCharLiteral(val) => val.to_owned(),
            IdlValueExpr::StringLiteral(val) => val.to_owned(),
            IdlValueExpr::WideStringLiteral(val) => val.to_owned(),
            IdlValueExpr::BooleanLiteral(val) => val.to_string(),
            IdlValueExpr::UnaryOp(op, expr) => format!("{}{}", op.to_str(), expr.to_string()),
            IdlValueExpr::BinaryOp(op, expr) => format!("{}{}", op.to_str(), expr.to_string()),
            IdlValueExpr::Expr(expr1, expr2) => format!("{}{}", expr1.to_string(), expr2.to_string()),
            IdlValueExpr::Brace(expr) => format!("({})", expr.to_string()),
            IdlValueExpr::FloatLiteral(integral, fraction, exponent, suffix) => {
                format!("{}.{}e{}{}", integral.as_ref().unwrap().clone(), fraction.as_ref().unwrap().clone(), exponent.as_ref().unwrap().clone(), suffix.as_ref().unwrap().clone())
            },
            IdlValueExpr::ScopedName(name) => name.to_string(),
        }
    }
}

///
impl Default for IdlValueExpr {
    fn default() -> IdlValueExpr { IdlValueExpr::None }
}

///
#[derive(Clone, Debug)]
pub struct IdlStructMember {
    pub id: String,
    pub type_spec: Box<IdlTypeSpec>,
}

///
#[derive(Clone, Debug)]
pub struct IdlSwitchElement {
    pub id: String,
    pub type_spec: Box<IdlTypeSpec>,
}

///
#[derive(Clone, Debug)]
pub enum IdlSwitchLabel {
    Label(Box<IdlValueExpr>),
    Default,
}

///
#[derive(Clone, Debug)]
pub struct IdlSwitchCase {
    pub labels: Vec<IdlSwitchLabel>,
    pub elem_spec: Box<IdlSwitchElement>,
}

///
#[derive(Clone, Debug)]
pub enum IdlTypeSpec {
    None,
    ArrayType(Box<IdlTypeSpec>, Vec<Box<IdlValueExpr>>),
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

///
impl IdlTypeSpec {
    ///
    pub fn to_string(&self) -> String {
        match self {
            IdlTypeSpec::F32Type => "f32".to_string(),
            IdlTypeSpec::F64Type => "f64".to_string(),
            IdlTypeSpec::F128Type => "f128".to_string(),
            IdlTypeSpec::I16Type => "i16".to_string(),
            IdlTypeSpec::I32Type => "i32".to_string(),
            IdlTypeSpec::I64Type => "i64".to_string(),
            IdlTypeSpec::U16Type => "u16".to_string(),
            IdlTypeSpec::U32Type => "u32".to_string(),
            IdlTypeSpec::U64Type => "u64".to_string(),
            IdlTypeSpec::CharType => "char".to_string(),
            IdlTypeSpec::WideCharType => "char".to_string(),
            IdlTypeSpec::BooleanType => "bool".to_string(),
            IdlTypeSpec::OctetType => "u8".to_string(),
            IdlTypeSpec::StringType(None) => "String".to_string(),
            IdlTypeSpec::WideStringType(None) => "String".to_string(),
            // TODO implement String/Sequence bounds
            IdlTypeSpec::StringType(_) => "String".to_string(),
            // TODO implement String/Sequence bounds for serializer and deserialzer
            IdlTypeSpec::WideStringType(_) => "String".to_string(),
            IdlTypeSpec::SequenceType(typ_expr) => {
                format!("Vec<{}>", typ_expr.as_ref().to_string())
            }
            IdlTypeSpec::ArrayType(typ_expr, dim_expr_list) => {
                let dim_list_str = dim_expr_list.into_iter().map(|expr| {
                    format!(";{}]", expr.to_string())
                }).collect::<String>();
                format!("{}{}{dim_list_str}", "[".repeat(dim_expr_list.len()), typ_expr.to_string())
            },
            IdlTypeSpec::ScopedName(name) => name.to_string(),
            _ => unimplemented!(),
        }
    }
}

///
impl Default for IdlTypeSpec {
    fn default() -> IdlTypeSpec { IdlTypeSpec::None }
}

///
#[derive(Clone, Debug)]
pub enum IdlTypeDclKind {
    None,
    TypeDcl(String, Box<IdlTypeSpec>),
    StructDcl(String, Vec<Box<IdlStructMember>>),
    UnionDcl(String, Box<IdlTypeSpec>, Vec<IdlSwitchCase>),
    EnumDcl(String,  Vec<String>),
}

///
impl Default for IdlTypeDclKind {
    fn default() -> IdlTypeDclKind { IdlTypeDclKind::None }
}

///
#[derive(Clone, Debug, Default)]
pub struct IdlTypeDcl(pub IdlTypeDclKind);

#[derive(Serialize)]
struct IdlStructField {
    name: String,
    type_str: String,
}

#[derive(Serialize)]
struct IdlSwitchField {
    name: String,
    element_id: String,
    element_type: String,
}

///
impl IdlTypeDcl {
    ///
    ///
    pub fn render(&mut self, env: &minijinja::Environment, level: usize) -> Result<String, minijinja::Error> {
        match self.0 {
            IdlTypeDclKind::TypeDcl(ref id, ref type_spec) => {
                let tmpl = env.get_template("typedef.j2")?;
                tmpl.render(
                    minijinja::context!{
                        typedef_name => id,
                        typedef_type => type_spec.to_string(),
                        indent_level => level
                    }
                )
            }
            IdlTypeDclKind::StructDcl(ref id, ref type_spec) => {
                let tmpl = env.get_template("struct.j2")?;
                let fields = type_spec
                    .into_iter()
                    .map(|field|
                        IdlStructField {
                            name: field.id.clone(),
                            type_str: field.type_spec.to_string()
                        }
                    ).collect::<Vec<IdlStructField>>();

                tmpl.render(
                    minijinja::context!{
                        struct_name => id,
                        fields,
                        indent_level => level
                    }
                )
            }
            IdlTypeDclKind::EnumDcl(ref id, ref enums) => {
                let tmpl = env.get_template("enum.j2")?;
                tmpl.render(
                    minijinja::context!{
                        enum_name => id,
                        variants => enums,
                        indent_level => level
                    }
                )
            }
            IdlTypeDclKind::UnionDcl(ref id, ref _type_spec, ref switch_cases) => {
                let tmpl = env.get_template("union_switch.j2")?;
                let union_members = switch_cases
                    .into_iter()
                    .map(|case|
                        case.labels.clone()
                            .into_iter()
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
                            }).collect::<Vec<IdlSwitchField>>()
                    ).flatten()
                    .collect::<Vec<IdlSwitchField>>();

                tmpl.render(
                    minijinja::context!{
                        union_name => id,
                        union_members,
                        indent_level => level
                    }
                )
            },
            IdlTypeDclKind::None => Ok(String::new())
        }
    }
}

///
#[derive(Clone, Default, Debug)]
pub struct IdlConstDcl {
    pub id: String,
    pub typedcl: Box<IdlTypeSpec>,
    pub value: Box<IdlValueExpr>,
}

///
impl IdlConstDcl {
    ///
    pub fn render(&mut self, env: &minijinja::Environment, level: usize) -> Result<String, minijinja::Error> {
        let tmpl = env.get_template("const.j2")?;
        tmpl.render(
            minijinja::context!{
                const_name => self.id,
                const_type => self.typedcl.to_string(),
                const_value => self.value.to_string(),
                indent_level => level
            }
        )
    }
}

///
#[derive(Clone,
Default, Debug)]
pub struct IdlModule {
    pub id: Option<String>,
    pub uses: Vec<String>,
    pub modules: LinkedHashMap<String, Box<IdlModule>>,
    pub types: LinkedHashMap<String, Box<IdlTypeDcl>>,
    pub constants: LinkedHashMap<String, Box<IdlConstDcl>>,
}

///
impl IdlModule {
    pub fn new(id: Option<String>) -> IdlModule {
        IdlModule {
            id: id,
            uses: Vec::new(),
            modules: LinkedHashMap::default(),
            types: LinkedHashMap::default(),
            constants: LinkedHashMap::default(),
        }
    }

    pub fn render(&mut self, env: &minijinja::Environment, level: usize) -> Result<String, minijinja::Error> {
        let mut module_info = String::new();
        let add = if self.id.is_some() { 1 } else { 0 };

        // TODO populate this instead of hardcoded solution
        let indent = level * INDENTION;
        for required_use in &self.uses {
            let uses = format!("{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{required_use}\n", "", "");
            module_info.push_str(&uses);
        }

        let use_vec = format!("{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{IMPORT_VEC}\n", "", "", indent = (level + add) * INDENTION);
        let use_serde = format!("{:indent$}{ATTR_ALLOW_UNUSED_IMPORTS}\n{:indent$}{IMPORT_SERDE}\n", "", "", indent = (level + add) * INDENTION);
        module_info.push_str(&use_vec);
        module_info.push_str(&use_serde);

        for typ in self.types.entries() {
            let rendered = typ.into_mut().render(&env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        for module in self.modules.entries() {
            let rendered = module.into_mut().render(&env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        for cnst in self.constants.entries() {
            let rendered = cnst.into_mut().render(&env, level + add)?;
            module_info.push_str(&rendered);
            module_info.push('\n');
        }

        match self.id {
            Some(ref id_str) => {
                let tmpl = env.get_template("module.j2")?;
                tmpl.render(
                    minijinja::context!{
                        module_name => id_str,
                        module_information => module_info,
                        indent_level => level
                    }
                )
            },
            None => Ok(module_info),
        }
    }
}
