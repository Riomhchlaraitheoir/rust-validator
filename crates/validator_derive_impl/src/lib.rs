use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{
    AngleBracketedGenericArguments,
    Arm,
    Attribute,
    Block,
    Data,
    DataEnum,
    DataStruct,
    DeriveInput,
    Expr,
    ExprStruct,
    ExprTuple,
    FieldMutability,
    FieldPat,
    Fields,
    FieldsNamed,
    FieldsUnnamed,
    FieldValue,
    FnArg,
    GenericArgument,
    ImplItem,
    ImplItemFn,
    ImplItemType,
    Index,
    Item,
    ItemEnum,
    ItemImpl,
    ItemStruct,
    LitInt,
    Member,
    Meta,
    parenthesized,
    parse_quote,
    Pat,
    Path,
    PathArguments,
    PathSegment,
    PatIdent,
    PatStruct,
    PatTupleStruct,
    PatType,
    ReturnType,
    Signature,
    Stmt,
    Token,
    Type,
    TypePath,
    TypeReference,
    TypeTuple,
    Variant,
    Visibility
};
use syn::parse::{Parse, Parser, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Colon, Comma, Fn, PathSep, Semi};

#[cfg(test)]
mod test;

fn validator_signature() -> Signature {
    Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Fn::default(),
        ident: Ident::new("validator", Span::call_site()),
        generics: Default::default(),
        paren_token: Default::default(),
        inputs: Default::default(),
        variadic: None,
        output: ReturnType::Type(Default::default(), Box::new(
            Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: [PathSegment {
                        ident: Ident::new("Self", Span::call_site()),
                        arguments: Default::default(),
                    }, PathSegment {
                        ident: Ident::new("Validator", Span::call_site()),
                        arguments: Default::default(),
                    }].into_iter().collect(),
                },
            })
        )),
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = input.parse().with_message("DeriveInput")?;
        let data = match &input.data {
            Data::Struct(data) => {
                parse_struct_input(data).with_message("failed to parse struct")?
            }
            Data::Enum(data) => {
                parse_enum_input(&input, data).with_message("failed to parse enum")?
            }
            Data::Union(data) => return Err(syn::Error::new(data.union_token.span, "Validator is not supported for unions"))
        };
        Ok(Self {
            vis: input.vis,
            name: input.ident,
            data,
        })
    }
}

pub fn derive(input: Input) -> TokenStream {
    let mut items = Vec::new();
    items.push(input.error_definition());
    items.extend(input.validator());
    let validator_ext = input.validate_impl();
    quote! {
        #(#items)*
        #validator_ext
    }
}

fn parse_enum_input(input: &DeriveInput, data: &DataEnum) -> syn::Result<InputData> {
    let variants: syn::Result<Vec<_>> = data.variants.iter().map(|variant| {
        let fields = if variant.fields == Fields::Unit {
            None
        } else {
            Some(variant.fields
                .clone()
                .try_into()
                .with_message("failed to parse fields")?)
        };
        Ok(EnumVariant {
            derived_type: input.ident.clone(),
            name: variant.ident.clone(),
            fields,
        })
    }).collect();
    let variants = variants?;
    Ok(InputData::Enum {
        variants,
    })
}

pub struct Input {
    vis: Visibility,
    name: Ident,
    data: InputData,
}

enum InputData {
    Struct {
        fields: StructFields,
        semi_token: Option<Semi>,
    },
    Enum {
        variants: Vec<EnumVariant>
    },
}

struct StructFields {
    fields: Vec<Field>,
    named_fields: bool,
}

impl StructFields {
    fn validator_fields(&self) -> Fields {
        let define_validator_fields = self.fields.iter().map(Field::define_validator_field);
        if self.named_fields {
            named_fields(define_validator_fields)
        } else {
            unnamed_fields(define_validator_fields)
        }
    }

    fn error_definition(&self) -> Fields {
        let fields = self.fields.iter()
            .map(Field::error_field)
            .collect();
        if self.named_fields {
            Fields::Named(FieldsNamed {
                brace_token: Default::default(),
                named: fields,
            })
        } else {
            Fields::Unnamed(FieldsUnnamed {
                paren_token: Default::default(),
                unnamed: fields,
            })
        }
    }
}

impl TryFrom<Fields> for StructFields {
    type Error = syn::Error;
    fn try_from(value: Fields) -> Result<Self, Self::Error> {
        match value {
            Fields::Unit => Err(syn::Error::new(value.span(), "Validator is not supported for unit structs")),
            Fields::Named(fields) => {
                let fields = fields.named.iter().cloned().map(|field| {
                    let vis = field.vis;
                    let name = field.ident.unwrap();
                    let ty = field.ty;
                    let validator = validator_from_attrs(field.attrs).with_message("failed to parse validator from attrs")?;
                    Ok(Field {
                        name: Member::Named(name),
                        ty,
                        vis,
                        validator,
                    })
                }).collect::<syn::Result<Vec<_>>>().with_message("failed to parse fields")?;
                Ok(Self {
                    fields,
                    named_fields: true,
                })
            }
            Fields::Unnamed(fields) => {
                let fields = fields.unnamed.iter().cloned().enumerate().map(|(i, field)| {
                    let vis = field.vis;
                    let ty = field.ty;
                    let validator = validator_from_attrs(field.attrs).with_message("failed to parse validator from attrs")?;
                    Ok(Field {
                        name: Member::Unnamed(Index::from(i)),
                        ty,
                        vis,
                        validator,
                    })
                }).collect::<syn::Result<Vec<_>>>().with_message("failed to parse fields")?;
                Ok(Self {
                    fields,
                    named_fields: false,
                })
            }
        }
    }
}

struct EnumVariant {
    derived_type: Ident,
    name: Ident,
    fields: Option<StructFields>,
}


struct Field {
    name: Member,
    ty: Type,
    vis: Visibility,
    validator: Validator,
}

#[derive(Debug, Default)]
enum Validator {
    NotEmpty,
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Email,
    Url,
    IpAddr,
    Length(Option<usize>, Option<usize>),
    Elements(Box<Self>),
    #[default]
    Default,
    Ignore,
    Tuple(Vec<Self>),
}

fn parse_struct_input(data: &DataStruct) -> syn::Result<InputData> {
    Ok(InputData::Struct {
        fields: data.fields.clone().try_into()?,
        semi_token: data.semi_token,
    })
}

impl Input {
    fn validator_type(&self) -> Ident {
        Ident::new(&format!("{name}Validator", name = self.name), self.name.span())
    }
    fn error_type(&self) -> Ident {
        Ident::new(&format!("{name}ValidationErrors", name = self.name), self.name.span())
    }

    fn error_definition(&self) -> Item {
        let Input { vis, name: _, data } = self;
        let error_type = self.error_type();
        match data {
            InputData::Struct { fields, semi_token } => {
                let fields = fields.error_definition();
                Item::Struct(
                    ItemStruct {
                        attrs: vec![
                            parse_quote!(#[derive(Debug, PartialEq, Clone)])
                        ],
                        vis: vis.clone(),
                        struct_token: Default::default(),
                        ident: error_type,
                        generics: Default::default(),
                        fields,
                        semi_token: semi_token.as_ref().cloned(),
                    },
                )
            }
            InputData::Enum { variants } => {
                Item::Enum(
                    ItemEnum {
                        attrs: vec![parse_quote!(#[derive(Debug, PartialEq, Clone)])],
                        vis: self.vis.clone(),
                        enum_token: Default::default(),
                        ident: error_type,
                        generics: Default::default(),
                        brace_token: Default::default(),
                        variants: variants.iter()
                            .filter_map(EnumVariant::error_variant)
                            .collect(),
                    },
                )
            }
        }
    }
    fn validator(&self) -> Vec<Item> {
        let Input { data, .. } = self;
        match data {
            InputData::Struct { fields, semi_token } => {
                let derived_type = &self.name;
                let name = self.validator_type();
                let vis = &self.vis;
                let error = self.error_type();
                let define_validator_fields = fields.validator_fields();
                let validate_fields = fields.fields.iter().map(Field::validate_field).collect();

                let error_declaration = Expr::Struct(ExprStruct {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [PathSegment {
                            ident: error.clone(),
                            arguments: Default::default(),
                        }].into_iter().collect(),
                    },
                    brace_token: Default::default(),
                    fields: validate_fields,
                    dot2_token: None,
                    rest: None,
                });

                vec![
                    Item::Struct(ItemStruct {
                        attrs: vec![],
                        vis: vis.clone(),
                        struct_token: Default::default(),
                        ident: name.clone(),
                        generics: Default::default(),
                        fields: define_validator_fields,
                        semi_token: semi_token.as_ref().cloned(),
                    }),
                    Item::Impl(ItemImpl {
                        attrs: vec![],
                        defaultness: None,
                        unsafety: None,
                        impl_token: Default::default(),
                        generics: Default::default(),
                        trait_: Some((None, parse_quote!(::validator::Validator<#derived_type>), Default::default())),
                        self_ty: Box::new(simple_type(name)),
                        brace_token: Default::default(),
                        items: vec![
                            ImplItem::Type(ImplItemType {
                                attrs: vec![],
                                vis: Visibility::Inherited,
                                defaultness: None,
                                type_token: Default::default(),
                                ident: Ident::new("Error", Span::call_site()),
                                generics: Default::default(),
                                eq_token: Default::default(),
                                ty: simple_type(error.clone()),
                                semi_token: Default::default(),
                            }),
                            ImplItem::Fn(ImplItemFn {
                                attrs: vec![],
                                vis: Visibility::Inherited,
                                defaultness: None,
                                sig: Signature {
                                    constness: None,
                                    asyncness: None,
                                    unsafety: None,
                                    abi: None,
                                    fn_token: Default::default(),
                                    ident: Ident::new("validate", Span::call_site()),
                                    generics: Default::default(),
                                    paren_token: Default::default(),
                                    inputs: [
                                        parse_quote!(&self),
                                        FnArg::Typed(PatType {
                                            attrs: vec![],
                                            pat: Box::new(if fields.named_fields {
                                                Pat::Struct(PatStruct {
                                                    attrs: vec![],
                                                    qself: None,
                                                    path: Path {
                                                        leading_colon: None,
                                                        segments: [
                                                            PathSegment {
                                                                ident: self.name.clone(),
                                                                arguments: Default::default(),
                                                            }
                                                        ].into_iter().collect(),
                                                    },
                                                    brace_token: Default::default(),
                                                    fields: fields.fields.iter()
                                                        .map(Field::field_pat)
                                                        .collect(),
                                                    rest: None,
                                                })
                                            } else {
                                                Pat::TupleStruct(PatTupleStruct {
                                                    attrs: vec![],
                                                    qself: None,
                                                    path: Path {
                                                        leading_colon: None,
                                                        segments: [
                                                            PathSegment {
                                                                ident: self.name.clone(),
                                                                arguments: Default::default(),
                                                            }
                                                        ].into_iter().collect(),
                                                    },
                                                    paren_token: Default::default(),
                                                    elems: fields.fields.iter()
                                                        .map(Field::pat)
                                                        .collect(),
                                                })
                                            }),
                                            colon_token: Default::default(),
                                            ty: Box::new(Type::Reference(TypeReference {
                                                and_token: Default::default(),
                                                lifetime: None,
                                                mutability: None,
                                                elem: Box::new(simple_type(self.name.clone())),
                                            })),
                                        })
                                    ].into_iter().collect(),
                                    variadic: None,
                                    output: parse_quote!(-> Result<(), Self::Error>),
                                },
                                block: parse_quote!(
                                    {
                                        let mut _valid = true;
                                        let validator = self;
                                        let error = #error_declaration;
                                        if _valid {
                                            Ok(())
                                        } else {
                                            Err(error)
                                        }
                                    }
                                ),
                            }),
                        ],
                    })]
            }
            InputData::Enum { variants } => {
                let derived_type = self.name.clone();
                let match_arms = variants.iter()
                    .enumerate().map(
                    |(index, EnumVariant { name, fields, .. })| -> Arm {
                        let Some(fields) = fields.as_ref() else {
                            return parse_quote!(#derived_type::#name => Ok(()));
                        };
                        let error_declaration = Expr::Struct(ExprStruct {
                            attrs: vec![],
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: [PathSegment {
                                    ident: self.error_type(),
                                    arguments: Default::default(),
                                }, PathSegment {
                                    ident: name.clone(),
                                    arguments: Default::default(),
                                }].into_iter().collect(),
                            },
                            brace_token: Default::default(),
                            fields: fields.fields.iter()
                                .map(Field::validate_field)
                                .collect(),
                            dot2_token: None,
                            rest: None,
                        }
                        );

                        let index: Index = index.into();

                        Arm {
                            attrs: vec![],
                            pat: if fields.named_fields {
                                Pat::Struct(PatStruct {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: [
                                            PathSegment {
                                                ident: derived_type.clone(),
                                                arguments: Default::default(),
                                            },
                                            PathSegment {
                                                ident: name.clone(),
                                                arguments: Default::default(),
                                            }
                                        ].into_iter().collect(),
                                    },
                                    brace_token: Default::default(),
                                    fields: fields.fields
                                        .iter()
                                        .map(Field::field_pat)
                                        .collect(),
                                    rest: None,
                                })
                            } else {
                                Pat::TupleStruct(PatTupleStruct {
                                    attrs: vec![],
                                    qself: None,
                                    path: Path {
                                        leading_colon: None,
                                        segments: [PathSegment {
                                            ident: self.name.clone(),
                                            arguments: Default::default(),
                                        }, PathSegment {
                                            ident: name.clone(),
                                            arguments: Default::default(),
                                        }].into_iter().collect(),
                                    },
                                    paren_token: Default::default(),
                                    elems: fields.fields
                                        .iter()
                                        .map(Field::pat)
                                        .collect(),
                                })
                            },
                            guard: None,
                            fat_arrow_token: Default::default(),
                            body: Box::new(parse_quote!({
                                let mut _valid = true;
                                let validator = &self.#index;
                                let error = #error_declaration;
                                if _valid {
                                    Ok(())
                                } else {
                                    Err(error)
                                }
                            })),
                            comma: Default::default(),
                        }
                    });

                let mut items: Vec<_> = variants.iter()
                    .filter_map(EnumVariant::validator)
                    .collect();

                items.push(Item::Struct(ItemStruct {
                    attrs: vec![],
                    vis: self.vis.clone(),
                    struct_token: Default::default(),
                    ident: self.validator_type(),
                    generics: Default::default(),
                    fields: Fields::Unnamed(FieldsUnnamed {
                        paren_token: Default::default(),
                        unnamed: variants.iter().filter_map(|variant| {
                            variant.fields.as_ref()?;
                            Some(syn::Field {
                                attrs: vec![],
                                vis: Visibility::Inherited,
                                mutability: FieldMutability::None,
                                ident: None,
                                colon_token: None,
                                ty: simple_type(variant.validator_name()),
                            })
                        }).collect(),
                    }),
                    semi_token: Some(Default::default()),
                }));

                items.push(Item::Impl(ItemImpl {
                    attrs: vec![],
                    defaultness: None,
                    unsafety: None,
                    impl_token: Default::default(),
                    generics: Default::default(),
                    trait_: Some((None, parse_quote!(::validator::Validator<#derived_type>), Default::default())),
                    self_ty: Box::new(simple_type(self.validator_type())),
                    brace_token: Default::default(),
                    items: vec![
                        ImplItem::Type(ImplItemType {
                            attrs: vec![],
                            vis: Visibility::Inherited,
                            defaultness: None,
                            type_token: Default::default(),
                            ident: Ident::new("Error", Span::call_site()),
                            generics: Default::default(),
                            eq_token: Default::default(),
                            ty: simple_type(self.error_type()),
                            semi_token: Default::default(),
                        }),
                        ImplItem::Fn(parse_quote! {
                                fn validate(&self, value: &#derived_type) -> Result<(), Self::Error> {
                                    match value {
                                        #(#match_arms),*
                                    }
                                }
                            }),
                    ],
                }));

                items
            }
        }
    }

    fn validate_impl(&self) -> Item {
        let Input { name: derived_type, data, .. } = self;
        match data {
            InputData::Struct { fields: StructFields { fields, .. }, .. } => {
                let create_validator = fields.iter()
                    .map(Field::create_validator)
                    .collect();
                let validator_type = self.validator_type();


                let validator_declaration = Expr::Struct(ExprStruct {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [PathSegment {
                            ident: validator_type.clone(),
                            arguments: Default::default(),
                        }].into_iter().collect(),
                    },
                    brace_token: Default::default(),
                    fields: create_validator,
                    dot2_token: None,
                    rest: None,
                });

                Item::Impl(ItemImpl {
                    attrs: vec![],
                    defaultness: None,
                    unsafety: None,
                    impl_token: Default::default(),
                    generics: Default::default(),
                    trait_: Some((
                        None,
                        parse_quote!(::validator::Validate),
                        Default::default()
                    )),
                    self_ty: Box::new(simple_type(derived_type.clone())),
                    brace_token: Default::default(),
                    items: vec![
                        ImplItem::Type(ImplItemType {
                            attrs: vec![],
                            vis: Visibility::Inherited,
                            defaultness: None,
                            type_token: Default::default(),
                            ident: Ident::new("Validator", Span::call_site()),
                            generics: Default::default(),
                            eq_token: Default::default(),
                            ty: simple_type(validator_type.clone()),
                            semi_token: Default::default(),
                        }),
                        ImplItem::Fn(ImplItemFn {
                            attrs: vec![],
                            vis: Visibility::Inherited,
                            defaultness: None,
                            sig: validator_signature(),
                            block: Block {
                                brace_token: Default::default(),
                                stmts: vec![
                                    Stmt::Expr(validator_declaration, None)
                                ],
                            },
                        }),
                    ],
                })
            }
            InputData::Enum { variants } => {
                Item::Impl(ItemImpl {
                    attrs: vec![],
                    defaultness: None,
                    unsafety: None,
                    impl_token: Default::default(),
                    generics: Default::default(),
                    trait_: Some((
                        None,
                        parse_quote!(::validator::Validate),
                        Default::default()
                    )),
                    self_ty: Box::new(simple_type(derived_type.clone())),
                    brace_token: Default::default(),
                    items: vec![
                        ImplItem::Type(ImplItemType {
                            attrs: vec![],
                            vis: Visibility::Inherited,
                            defaultness: None,
                            type_token: Default::default(),
                            ident: Ident::new("Validator", Span::call_site()),
                            generics: Default::default(),
                            eq_token: Default::default(),
                            ty: simple_type(self.validator_type()),
                            semi_token: Default::default(),
                        }),
                        ImplItem::Fn(ImplItemFn {
                            attrs: vec![],
                            vis: Visibility::Inherited,
                            defaultness: None,
                            sig: validator_signature(),
                            block: Block {
                                brace_token: Default::default(),
                                stmts: vec![
                                    Stmt::Expr(Expr::Struct(ExprStruct {
                                        attrs: vec![],
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: [PathSegment {
                                                ident: self.validator_type(),
                                                arguments: Default::default(),
                                            }].into_iter().collect(),
                                        },
                                        brace_token: Default::default(),
                                        fields: variants.iter()
                                            .filter_map(EnumVariant::create_validator)
                                            .enumerate()
                                            .map(|(i, expr)| {
                                                FieldValue {
                                                    attrs: vec![],
                                                    member: Member::Unnamed(i.into()),
                                                    colon_token: Some(Colon::default()),
                                                    expr,
                                                }
                                            }).collect(),
                                        dot2_token: None,
                                        rest: None,
                                    }), None)
                                ],
                            },
                        }),
                    ],
                })
            }
        }
    }
}

fn unnamed_fields(fields: impl Iterator<Item=syn::Field>) -> Fields {
    Fields::Unnamed(FieldsUnnamed {
        paren_token: Default::default(),
        unnamed: fields.collect(),
    })
}

fn named_fields(fields: impl Iterator<Item=syn::Field>) -> Fields {
    Fields::Named(FieldsNamed {
        brace_token: Default::default(),
        named: fields.collect(),
    })
}

fn simple_type(name: Ident) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: [PathSegment::from(name)].into_iter().collect(),
        },
    })
}

impl Field {
    fn pattern_name(&self) -> Ident {
        match &self.name {
            Member::Named(name) => name.clone(),
            Member::Unnamed(index) => Ident::new(&format!("value{}", index.index), index.span)
        }
    }

    fn field_pat(&self) -> FieldPat {
        FieldPat {
            attrs: vec![],
            member: self.name.clone(),
            colon_token: None,
            pat: Box::new(self.pat()),
        }
    }

    fn pat(&self) -> Pat {
        Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: self.pattern_name(),
            subpat: None,
        })
    }

    fn error_field(&self) -> syn::Field {
        let error_type = self.validator.error_type(&self.ty);
        let ty = parse_quote!(Option<#error_type>);
        self.field(ty)
    }

    fn define_validator_field(&self) -> syn::Field {
        self.field(self.validator.validator_type(&self.ty))
    }

    fn validate_field(&self) -> FieldValue {
        let name = self.name.clone();
        let value = self.pattern_name();
        let expr = parse_quote!(
                {
                    match validator.#name.validate(#value) {
                        Ok(()) => None,
                        Err(error) => {
                            _valid = false;
                            Some(error)
                        }
                    }
                }
            );
        self.field_value(expr)
    }

    fn create_validator(&self) -> FieldValue {
        self.field_value(self.validator.create(&self.ty))
    }

    fn field(&self, ty: Type) -> syn::Field {
        let (name, colon) = match &self.name {
            Member::Named(name) => (Some(name), Some(parse_quote!(:))),
            Member::Unnamed(_) => (None, None),
        };
        syn::Field {
            attrs: vec![],
            vis: self.vis.clone(),
            mutability: FieldMutability::None,
            ident: name.cloned(),
            colon_token: colon,
            ty,
        }
    }

    fn field_value(&self, expr: Expr) -> FieldValue {
        FieldValue {
            attrs: vec![],
            member: self.name.clone(),
            colon_token: Some(Default::default()),
            expr,
        }
    }
}

impl EnumVariant {
    fn validator_name(&self) -> Ident {
        let Self { derived_type, name, .. } = self;
        Ident::new(&format!("{derived_type}_{name}_Validator"), self.derived_type.span())
    }

    fn create_validator(&self) -> Option<Expr> {
        let fields = self.fields.as_ref()?;
        Some(Expr::Struct(ExprStruct {
            attrs: vec![],
            qself: None,
            path: Path {
                leading_colon: None,
                segments: [
                    PathSegment {
                        ident: self.validator_name(),
                        arguments: Default::default(),
                    }
                ].into_iter().collect(),
            },
            brace_token: Default::default(),
            fields: fields.fields
                .iter()
                .map(Field::create_validator)
                .collect(),
            dot2_token: None,
            rest: None,
        }))
    }

    fn validator(&self) -> Option<Item> {
        let fields = self.fields.as_ref()?;
        Some(Item::Struct(
            ItemStruct {
                attrs: vec![
                    parse_quote!(#[allow(non_camel_case_types)]),
                    parse_quote!(#[doc(hidden)]),
                ],
                vis: Visibility::Inherited,
                struct_token: Default::default(),
                ident: self.validator_name(),
                generics: Default::default(),
                fields: fields.validator_fields(),
                semi_token: if fields.named_fields { None } else { Some(Semi::default()) },
            }
        ))
    }
    fn error_variant(&self) -> Option<Variant> {
        let fields = self.fields.as_ref()?;
        Some(Variant {
            attrs: vec![],
            ident: self.name.clone(),
            fields: fields.error_definition(),
            discriminant: None,
        })
    }
}

fn validator_from_attrs(attrs: Vec<Attribute>) -> Result<Validator, syn::Error> {
    let attr: Vec<_> = attrs.iter().filter(|attr| {
        if let Meta::List(list) = &attr.meta {
            let path = &list.path.segments;
            path.len() == 1 && path[0].ident == "validator"
        } else {
            false
        }
    }).collect();
    if attr.len() > 1 {
        return Err(syn::Error::new(attr[0].span(), "validator attribute may only be used once on each field"));
    }
    let Some(&attr) = attr.first() else { return Ok(Validator::default()); };

    let Meta::List(list) = &attr.meta else { return Ok(Validator::default()); };
    Validator::parse.parse2(list.tokens.clone())
        .with_message("failed to parse validator")
}

impl Parse for Validator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse().with_message("failed to parse validator type")?;
        let val_type = ident.to_string();
        match val_type.as_str() {
            "not_empty" => Ok(Validator::NotEmpty),
            "email" => Ok(Validator::Email),
            "url" => Ok(Validator::Url),
            "ip" => Ok(Validator::IpAddr),
            "ignore" => Ok(Validator::Ignore),
            "elements" => Ok(Validator::Elements({
                let content;
                parenthesized!(content in input);
                Box::new(content.parse()?)
            })),
            "length" => {
                let content;
                parenthesized!(content in input);
                let mut equal_name = None;
                let mut equal = None;
                let mut min = None;
                let mut max = None;
                loop {
                    let name: Ident = content.parse().with_message("failed to parse length option name")?;
                    content.parse::<Token![=]>().with_message("failed to parse length option '=' token")?;
                    let value: LitInt = content.parse().with_message("failed to parse length option value")?;
                    let value: usize = value.base10_parse().map_err(|err| {
                        syn::Error::new(value.span(), format!("failed to parse usize: {err}"))
                    })?;
                    match name.to_string().as_str() {
                        "equal" => {
                            equal_name = Some(name);
                            equal = Some(value)
                        }
                        "min" => min = Some(value),
                        "max" => max = Some(value),
                        other => return Err(syn::Error::new(name.span(), format!(r#"unknown option: "{other}""#)))
                    }
                    if Comma::parse(&content).is_err() {
                        break;
                    }
                }

                if let Some(equal) = equal {
                    if min.is_some() || max.is_some() {
                        emit_error!(equal_name.unwrap().span(), "cannot use 'equal' with either 'min' or 'max'")
                    }

                    Ok(Validator::Length(Some(equal), Some(equal)))
                } else if min.is_some() || max.is_some() {
                    Ok(Validator::Length(min, max))
                } else {
                    Err(syn::Error::new(val_type.span(), "no options found, one of 'equal', 'min', 'max' must be set"))
                }
            }
            "and" | "or" => {
                let content;
                parenthesized!(content in input);
                let left = content.parse().with_message("failed to parse binary left")?;
                let _ = content.parse::<Token![,]>().with_message("failed to parse binary comma")?;
                let right = content.parse().with_message("failed to parse binary right")?;
                Ok(match val_type.as_str() {
                    "and" => Validator::And(Box::new(left), Box::new(right)),
                    "or" => Validator::Or(Box::new(left), Box::new(right)),
                    _ => unreachable!()
                })
            }
            "tuple" => {
                let content;
                parenthesized!(content in input);
                let mut children = Vec::new();
                loop {
                    let child = content.parse()
                        .with_message(&format!("failed to parse tuple child {i}", i = children.len()))?;
                    children.push(child);
                    if Comma::parse(&content).is_err() {
                        break;
                    }
                }
                Ok(Validator::Tuple(children))
            }
            other => {
                Err(syn::Error::new(ident.span(), format!("unknown validator type: \"{other}\"")))
            }
        }
    }
}

impl Validator {
    fn create(&self, ty: &Type) -> Expr {
        match self {
            Validator::NotEmpty => parse_quote!(::validator::NotEmptyValidator),
            Validator::And(left, right) => {
                let left = left.create(ty);
                let right = right.create(ty);
                parse_quote!(::validator::And::new(#left, #right))
            }
            Validator::Or(left, right) => {
                let left = left.create(ty);
                let right = right.create(ty);
                parse_quote!(::validator::Or::new(#left, #right))
            }
            Validator::Email => parse_quote!(::validator::EmailValidator),
            Validator::Url => parse_quote!(::validator::UrlValidator),
            Validator::IpAddr => parse_quote!(::validator::IpAddrValidator),
            Validator::Length(min, max) => {
                let min = option_literal(min.as_ref());
                let max = option_literal(max.as_ref());
                parse_quote!(::validator::LengthValidator::new(#min, #max))
            }
            Validator::Default => parse_quote!(<#ty as ::validator::Validate>::validator()),
            Validator::Elements(elements) => {
                let elements = elements.create(ty);
                parse_quote!(::validator::ElementsValidator::new(#elements))
            }
            Validator::Tuple(children) => {
                Expr::Tuple(ExprTuple {
                    attrs: vec![],
                    paren_token: Default::default(),
                    elems: children.iter()
                        .map(|child| {
                            child.create(ty)
                        }).collect(),
                })
            }
            Validator::Ignore => {
                parse_quote!(::validator::IgnoreValidator)
            }
        }
    }
    fn validator_type(&self, ty: &Type) -> Type {
        match self {
            Validator::NotEmpty => parse_quote!(::validator::NotEmptyValidator),
            Validator::And(left, right) => {
                let left = left.validator_type(ty);
                let right = right.validator_type(ty);
                parse_quote!(::validator::And<#left, #right>)
            }
            Validator::Or(left, right) => {
                let left = left.validator_type(ty);
                let right = right.validator_type(ty);
                parse_quote!(::validator::Or<#left, #right>)
            }
            Validator::Email => parse_quote!(::validator::EmailValidator),
            Validator::Url => parse_quote!(::validator::UrlValidator),
            Validator::IpAddr => parse_quote!(::validator::IpAddrValidator),
            Validator::Length(_, _) => parse_quote!(::validator::LengthValidator),
            Validator::Default => parse_quote!(<#ty as ::validator::Validate>::Validator),
            Validator::Elements(elements) => {
                let elements = elements.validator_type(ty);
                parse_quote!(::validator::ElementsValidator<#elements>)
            }
            Validator::Tuple(children) => {
                Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: children.iter()
                        .map(|child| {
                            child.validator_type(ty)
                        }).collect(),
                })
            }
            Validator::Ignore => {
                parse_quote!(::validator::IgnoreValidator)
            }
        }
    }
    fn error_type(&self, ty: &Type) -> Type {
        match self {
            Validator::NotEmpty => parse_quote!(::validator::EmptyValueError),
            Validator::And(left, right) => {
                let left = left.error_type(ty);
                let right = right.error_type(ty);
                parse_quote!(::validator::AndError<#left, #right>)
            }
            Validator::Or(left, right) => {
                let left = left.error_type(ty);
                let right = right.error_type(ty);
                parse_quote!((#left, #right))
            }
            Validator::Email => parse_quote!(::validator::InvalidEmailError),
            Validator::Url => parse_quote!(::validator::InvalidUrlError),
            Validator::IpAddr => parse_quote!(::std::net::AddrParseError),
            Validator::Length(_, _) => parse_quote!(::validator::InvalidLengthError),
            Validator::Default => parse_quote!(<<#ty as ::validator::Validate>::Validator as ::validator::Validator<#ty>>::Error),
            Validator::Elements(elements) => {
                let elements = elements.error_type(ty);
                parse_quote!(::validator::ElementsInvalid<#elements>)
            }
            Validator::Tuple(children) => {
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Default::default(),
                        segments: [
                            PathSegment {
                                ident: Ident::new("validator", Span::call_site()),
                                arguments: Default::default(),
                            },
                            PathSegment {
                                ident: Ident::new(&format!("TupleError{}", children.len()), Span::call_site()),
                                arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Default::default(),
                                    args: children.iter()
                                        .map(|child| {
                                            GenericArgument::Type(child.error_type(ty))
                                        }).collect(),
                                    gt_token: Default::default(),
                                }),
                            }
                        ].into_iter().collect(),
                    },
                })
            }
            Validator::Ignore => {
                parse_quote!(::core::convert::Infallible)
            }
        }
    }
}

fn option_literal<T: ToTokens>(opt: Option<T>) -> TokenStream {
    match opt {
        None => quote! { None },
        Some(value) => quote! { Some(#value) }
    }
}

#[allow(dead_code)]
fn option_type(inner: Type) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: Some(PathSep::default()),
            segments: [
                PathSegment {
                    ident: Ident::new("core", Span::call_site()),
                    arguments: Default::default(),
                },
                PathSegment {
                    ident: Ident::new("option", Span::call_site()),
                    arguments: Default::default(),
                },
                PathSegment {
                    ident: Ident::new("Option", Span::call_site()),
                    arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Default::default(),
                        args: [
                            GenericArgument::Type(inner)
                        ].into_iter().collect(),
                        gt_token: Default::default(),
                    }),
                }
            ].into_iter().collect(),
        },
    })
}

trait WithMessage {
    fn with_message(self, msg: &str) -> Self;
}

impl<T> WithMessage for syn::Result<T> {
    #[cfg(test)]
    fn with_message(self, msg: &str) -> Self {
        self.map_err(|err| {
            syn::Error::new(err.span(), format!("{msg}: {inner}", inner = err.to_string()))
        })
    }
    #[cfg(not(test))]
    fn with_message(self, _msg: &str) -> Self {
        self
    }
}
