use std::fmt::Display;

use syn::{Fields, GenericArgument, ItemEnum, PathArguments, Type};

pub enum TsType {
    String,
    Number,
    Boolean,
    Array(Box<TsType>),
    ArrayTypes(Vec<TsType>),
    Optional(Box<TsType>),
    Ident(String),
    StringLiteral(String),
    Object(Vec<TSField>),
    Union(Vec<TsType>),
}

impl Display for TsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TsType::String => "string".to_string(),
            TsType::Number => "number".to_string(),
            TsType::Boolean => "boolean".to_string(),
            TsType::Ident(id) => id.clone(),
            TsType::StringLiteral(lit) => format!("\"{}\"", lit),
            TsType::Object(fields) => {
                let fields_str: String = fields
                    .iter()
                    .flat_map(|e| format!("    {}\n", e).chars().collect::<Vec<_>>())
                    .collect();

                format!("{{\n{}}}", fields_str)
            }
            TsType::Union(types) => {
                let s = types.iter().map(|e| format!("{}", e)).collect::<Vec<_>>();
                s.join(" | ")
            }
            TsType::Array(t) => {
                format!("{}[]", t)
            }
            TsType::ArrayTypes(t) => {
                let inner = t
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                format!("[{}]", inner)
            }
            TsType::Optional(t) => {
                format!("{}", t)
            }
        };

        write!(f, "{}", s)
    }
}

pub struct TSField {
    pub name: String,
    pub optional: bool,
    pub ts_type: TsType,
}

impl Display for TSField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let optional_string = if self.optional { "?" } else { "" };
        write!(f, "{}{}: {},", self.name, optional_string, self.ts_type)
    }
}

pub struct TsTopType {
    pub name: String,
    pub typ: TsType,
}

impl TsTopType {
    pub fn new(name: String, typ: TsType) -> Self {
        Self { name, typ }
    }

    // pub fn from_rust_struct(name: String, fields: &syn::Fields) -> Self {
    //     let ts_fields = parse_fields(&fields);
    //     Self {
    //         name,
    //         typ: TsType::Object(ts_fields),
    //     }
    // }
}

impl Display for TsTopType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.typ {
            TsType::Object(obj) => {
                let interface_block_open = format!("interface {} {{\n", self.name);
                let interface_block_close = "}";
                let fields_def = obj.iter().fold("".to_owned(), |acc, field| {
                    format!("{}    {}\n", acc, field)
                });
                write!(
                    f,
                    "export {}{}{}",
                    interface_block_open, &fields_def, interface_block_close
                )
            }
            _ => {
                write!(f, "export type {} = {};", self.name, self.typ)
            }
        }
    }
}

pub struct Converter {
    path_overwriters: Vec<Box<dyn Fn(&Type) -> Option<TsType>>>,
}

impl Default for Converter {
    fn default() -> Self {
        Self {
            path_overwriters: Vec::new(),
        }
    }
}

impl Converter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_overwriter(&mut self, overwriter: Box<dyn Fn(&Type) -> Option<TsType>>) {
        self.path_overwriters.push(overwriter);
    }

    pub fn parse_from_file(&self, contents: &str) -> Vec<TsTopType> {
        let syntax = syn::parse_file(contents).unwrap();

        let mut result = Vec::new();
        for item in syntax.items.iter() {
            match item {
                syn::Item::Struct(ref item) => {
                    let ts_fields = self.parse_fields(&item.fields);

                    let interface =
                        TsTopType::new(item.ident.to_string(), TsType::Object(ts_fields));
                    result.push(interface);
                }
                syn::Item::Enum(en) => match Self::determine_enum_conv_type(en) {
                    EnumConvType::StringUnion => {
                        let variants = en
                            .variants
                            .iter()
                            .map(|e| TsType::StringLiteral(e.ident.to_string()));

                        result.push(TsTopType::new(
                            en.ident.to_string(),
                            TsType::Union(variants.collect()),
                        ));
                    }
                    EnumConvType::Interfaces => {
                        let res = en
                            .variants
                            .iter()
                            .map(|e| {
                                let mut fields = self.parse_fields(&e.fields);
                                fields.insert(
                                    0,
                                    TSField {
                                        name: "kind".to_string(),
                                        ts_type: TsType::StringLiteral(e.ident.to_string()),
                                        optional: false,
                                    },
                                );
                                TsType::Object(fields)
                            })
                            .collect::<Vec<_>>();
                        // let res = Vec::new();

                        result.push(TsTopType::new(en.ident.to_string(), TsType::Union(res)));
                    }
                },
                _ => {
                    // TODO: handle more types such as enums
                    continue;
                }
            }
        }

        result
    }

    fn determine_enum_conv_type(en: &ItemEnum) -> EnumConvType {
        for variant in &en.variants {
            match variant.fields {
                Fields::Unit => {
                    continue;
                }
                _ => return EnumConvType::Interfaces,
            }
        }

        EnumConvType::StringUnion
    }

    fn parse_fields(&self, fields: &Fields) -> Vec<TSField> {
        let mut ts_fields: Vec<TSField> = Vec::new();

        match fields {
            Fields::Named(ref fields) => {
                for field in fields.named.iter() {
                    let field_name =
                        to_camelcase(false, &field.ident.as_ref().unwrap().to_string());
                    if let Some(ts_type) = self.rust_type_to_ts_type(&field.ty) {
                        ts_fields.push(TSField {
                            name: field_name,
                            optional: matches!(ts_type, TsType::Optional(_)),
                            ts_type,
                        });
                    }
                }
            }
            Fields::Unnamed(ref fields) => {
                let mut ts_types: Vec<TsType> = Vec::new();

                for field in fields.unnamed.iter() {
                    // let field_name =
                    // to_camelcase(false, &field.ident.as_ref().unwrap().to_string());
                    if let Some(ts_type) = self.rust_type_to_ts_type(&field.ty) {
                        // ts_fields.push(TSField {
                        //     name: field_name,
                        //     optional: matches!(ts_type, TsType::Optional(_)),
                        //     ts_type,
                        // });
                        ts_types.push(ts_type);
                    }
                }

                match ts_types.len() {
                    1 => {
                        let ts_type = ts_types.pop().unwrap();

                        ts_fields.push(TSField {
                            name: "value".to_string(),
                            optional: matches!(ts_type, TsType::Optional(_)),
                            ts_type,
                        });
                    }
                    2.. => {
                        todo!();
                    }
                    _ => {}
                }
            }
            _ => {
                todo!();
            }
        }

        ts_fields
    }

    fn rust_type_to_ts_type(&self, rs_type: &Type) -> Option<TsType> {
        match &rs_type {
            Type::Path(type_path) => {
                let last = type_path.path.segments.last().unwrap();
                let field_type = last.ident.to_string();

                match field_type.as_str() {
                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "i128"
                    | "u128" | "isize" | "usize" | "f32" | "f64" => Some(TsType::Number),
                    "String" => Some(TsType::String),
                    "bool" => Some(TsType::Boolean),
                    "Vec" => Some(TsType::Array(Box::new(
                        self.extract_generic_type(&last.arguments).unwrap(),
                    ))),
                    "Option" => Some(TsType::Optional(Box::new(
                        self.extract_generic_type(&last.arguments).unwrap(),
                    ))),
                    "Box" => Some(self.extract_generic_type(&last.arguments).unwrap()),
                    _ => {
                        for ow in &self.path_overwriters {
                            if let Some(res) = ow(rs_type) {
                                return Some(res);
                            }
                        }

                        Some(TsType::Ident(field_type))
                    }
                }
            }
            _ => {
                // println!("{:?}", &field.ty);
                None
            }
        }
    }

    fn extract_generic_type(&self, args: &PathArguments) -> Option<TsType> {
        if let PathArguments::AngleBracketed(brackets) = args {
            let t = brackets.args.iter().find_map(|e| match e {
                GenericArgument::Type(t) => Some(t),
                _ => None,
            });

            if let Some(t) = t {
                self.rust_type_to_ts_type(t)
            } else {
                None
            }
        } else {
            None
        }
    }
}

enum EnumConvType {
    StringUnion,
    Interfaces,
}

fn to_camelcase(first_capital: bool, s: &str) -> String {
    let mut new_str = String::new();

    let mut capitalise_next = first_capital;
    let mut first = true;
    for ch in s.chars() {
        match ch {
            '_' if first => {
                new_str.push('_');
            }
            '_' => {
                capitalise_next = true;
            }
            _ => {
                first = false;
                if capitalise_next {
                    new_str.push(ch.to_uppercase().next().unwrap());
                    capitalise_next = false;
                } else {
                    new_str.push(ch)
                }
            }
        }
    }

    new_str
}
