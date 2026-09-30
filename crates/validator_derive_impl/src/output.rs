use crate::{EnumVariant, Field, Input, InputData, StructFields, Validator};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{ExprRange, Index, Member, RangeLimits};

impl StructFields {
    fn validator_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(Field::define_validator_field);
        if self.named_fields {
            quote!{{#(#fields),*}}
        } else {
            quote!((#(#fields),*))
        }
    }

    fn error_definition(&self) -> TokenStream {
        let fields = self.fields.iter().map(Field::error_field);
        if self.named_fields {
            quote!{{
                #(#fields),*
            }}
        } else {
            quote!((
                #(#fields),*
            ))
        }
    }
}

impl Input {
    pub fn generate_output(&self) -> TokenStream {
        let mut output = TokenStream::new();
        output.extend(self.error_definition());
        output.extend(self.validator());
        output.extend(self.validate_impl());
        output
    }

    fn validator_type(&self) -> Ident {
        format_ident!("{}Validator", self.name)
    }
    fn error_type(&self) -> Ident {
        format_ident!("{}ValidationErrors", self.name)
    }

    fn error_definition(&self) -> TokenStream {
        let Input { vis, name: _, data } = self;
        let error_type = self.error_type();
        match data {
            InputData::Struct { fields, .. } => {
                if fields.named_fields {
                    let members = fields.fields.iter().map(|f| &f.name);
                    let types = fields.fields.iter().map(|f| f.validator.error_type(&f.ty.to_token_stream()));
                    let field_vis = fields.fields.iter().map(|f| &f.vis);
                    quote! {
                        #[derive(Debug, PartialEq, Clone, Default)]
                        #vis struct #error_type {
                            #(#field_vis #members: Option<#types>,)*
                        }
                    }
                } else {
                    let types = fields.fields.iter().map(|f| f.validator.error_type(&f.ty.to_token_stream()));
                    quote! {
                        #[derive(Debug, PartialEq, Clone, Default)]
                        #vis struct #error_type(#(Option<#types>,)*);
                    }
                }
            }
            InputData::Enum { variants } => {
                let variants = variants.iter().filter_map(EnumVariant::error_variant);
                quote! {
                    #[derive(Debug, PartialEq, Clone)]
                    #vis enum #error_type { #(#variants),* }
                }
            }
        }
    }
    fn validator(&self) -> TokenStream {
        let Input { data, .. } = self;
        match data {
            InputData::Struct { fields, semi_token } => {
                let derived_type = &self.name;
                let name = self.validator_type();
                let vis = &self.vis;
                let error = self.error_type();
                let define_validator_fields = fields.validator_fields();
                let validate_fields = fields.fields.iter().map(Field::validate_field);
                let field_names = fields.fields.iter().map(|f| &f.name);

                let error_declaration = quote! {
                    #error {
                        #(#field_names: #validate_fields),*
                    }
                };

                let data_pat: TokenStream = if fields.named_fields {
                    let fields = fields.fields.iter().map(Field::field_pat);
                    quote!(#derived_type { #(#fields),* })
                } else {
                    let fields = fields.fields.iter().map(Field::pattern_name);
                    quote!(#derived_type ( #(#fields),* ))
                };

                quote! {
                    #vis struct #name #define_validator_fields #semi_token

                    impl ::validator::Validator<#derived_type> for #name {
                        type Error = #error;
                        #[allow(non_shorthand_field_patterns)]
                        fn validate(&self, #data_pat: &#derived_type) -> Result<(), Self::Error> {
                                let mut _valid = true;
                                let validator = self;
                                let error = #error_declaration;
                                if _valid {
                                    Ok(())
                                } else {
                                    Err(error)
                                }
                            }
                    }
                }
            }
            InputData::Enum { variants } => {
                let derived_type = self.name.clone();
                let match_arms =
                    variants
                        .iter()
                        .enumerate()
                        .map(|(index, EnumVariant { name, fields, .. })| {
                            let Some(fields) = fields.as_ref() else {
                                return quote!(#derived_type::#name => Ok(()));
                            };
                            let members = fields.fields.iter().map(|f| &f.name);
                            let error_type = self.error_type();
                            let error_fields = fields.fields.iter().map(Field::validate_field);

                            let fields = if fields.named_fields {
                                let fields = fields.fields.iter().map(Field::field_pat);
                                quote!({ #(#fields),* })
                            } else {
                                let fields = fields.fields.iter().map(Field::pattern_name);
                                quote!(( #(#fields),* ) )
                            };
                            let index = Index::from(index);

                            quote! {
                                #derived_type::#name #fields => {
                                    let mut _valid = true;
                                    let validator = &self.#index;
                                    let error = #error_type::#name { #(#members: #error_fields),* };
                                    if _valid {
                                        Ok(())
                                    } else {
                                        Err(error)
                                    }
                                }
                            }
                        });

                let variant_validators = variants.iter().filter_map(EnumVariant::validator);

                let validator_type = self.validator_type();
                let error_type = self.error_type();

                let variant_validator_names = variants.iter().filter_map(|variant| {
                    variant.fields.as_ref()?;
                    Some(variant.validator_name())
                });

                quote! {
                    #(#variant_validators)*

                    struct #validator_type(#(#variant_validator_names),*);

                    impl ::validator::Validator<#derived_type> for #validator_type {
                        type Error = #error_type;
                        fn validate(&self, value: &#derived_type) -> Result<(), Self::Error> {
                            match value {
                                #(#match_arms),*
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_impl(&self) -> TokenStream {
        let Input {
            name: derived_type,
            data,
            ..
        } = self;
        match data {
            InputData::Struct {
                fields: StructFields { fields, .. },
                ..
            } => {
                let create_validator = fields.iter().map(Field::create_validator);
                let validator_type = self.validator_type();
                let members = fields.iter().map(|f| &f.name);

                quote! {
                    impl ::validator::Validate for #derived_type {
                        type Validator = #validator_type;
                            fn validator() -> Self::Validator {
                                #validator_type {
                                    #(#members: #create_validator),*
                                }
                            }
                    }
                }
            }
            InputData::Enum { variants } => {
                let validator_type = self.validator_type();
                let create_validator = variants.iter().filter_map(EnumVariant::create_validator);
                let members = (0..variants.len()).map(syn::Index::from);
                quote! {
                    impl ::validator::Validate for #derived_type {
                        type Validator = #validator_type;
                        fn validator() -> Self::Validator {
                            #validator_type {
                                #(#members: #create_validator),*
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Field {
    fn pattern_name(&self) -> Ident {
        use syn::Member;
        match &self.name {
            Member::Named(name) => name.clone(),
            Member::Unnamed(index) => Ident::new(&format!("value{}", index.index), index.span),
        }
    }

    fn field_pat(&self) -> TokenStream {
        let name = &self.name;
        let pat = self.pattern_name();
        quote!(#name : #pat)
    }

    fn error_field(&self) -> TokenStream {
        let error_type = self.validator.error_type(&self.ty.to_token_stream());
        let ty = quote!(Option<#error_type>);
        self.field(ty)
    }

    fn define_validator_field(&self) -> TokenStream {
        self.field(self.validator.validator_type(&self.ty.to_token_stream()))
    }

    fn validate_field(&self) -> TokenStream {
        let name = self.name.clone();
        let field = self.pattern_name();
        let other = match &self.validator {
            Validator::Matches(Member::Named(name)) => Some(name.clone()),
            Validator::Matches(Member::Unnamed(index)) => Some(format_ident!("value{}", index.index)),
            _ => None
        };
        let args = if let Some(other) = other {
            quote!(&(#field, #other))
        } else {
            quote!(#field)
        };
        quote!(
            {
                match validator.#name.validate(#args) {
                    Ok(()) => None,
                    Err(error) => {
                        _valid = false;
                        Some(error)
                    }
                }
            }
        )
    }

    fn create_validator(&self) -> TokenStream {
        self.validator.create(&self.label, &self.ty.to_token_stream())
    }

    fn field(&self, ty: TokenStream) -> TokenStream {
        use syn::Member;
        match &self.name {
            Member::Named(name) => quote! {
                #name: #ty
            },
            Member::Unnamed(_) => quote! {
                #ty
            },
        }
    }
}

impl EnumVariant {
    fn validator_name(&self) -> Ident {
        let Self {
            derived_type, name, ..
        } = self;
        format_ident!("{derived_type}{name}Validator")
    }

    fn create_validator(&self) -> Option<TokenStream> {
        let fields = self.fields.as_ref()?;

        let validator_name = self.validator_name();
        let members = fields.fields.iter().map(|f| &f.name);
        let fields = fields.fields.iter().map(Field::create_validator);

        Some(quote! {
            #validator_name {
                #(#members: #fields),*
            }
        })
    }

    fn validator(&self) -> Option<TokenStream> {
        let fields = self.fields.as_ref()?;
        let validator_name = self.validator_name();
        let validator_fields = fields.fields.iter().map(Field::define_validator_field);
        let validator_fields = if fields.named_fields {
            quote!{{#(#validator_fields),*}}
        } else {
            quote!((#(#validator_fields),*);)
        };
        Some(quote! {
            #[doc(hidden)]
            struct #validator_name #validator_fields
        })
    }
    fn error_variant(&self) -> Option<TokenStream> {
        let fields = self.fields.as_ref()?;
        let name = &self.name;
        let fields = fields.error_definition();
        Some(quote! {
            #name #fields
        })
    }
}

impl Validator {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        match self {
            Validator::NotEmpty => quote!(::validator::NotEmptyValidator::new(#field)),
            Validator::And(left, right) => {
                let left = left.create(field, ty);
                let right = right.create(field, ty);
                quote!(::validator::And::new(#left, #right))
            }
            Validator::Or(left, right) => {
                let left = left.create(field, ty);
                let right = right.create(field, ty);
                quote!(::validator::Or::new(#left, #right))
            }
            Validator::Email => quote!(::validator::EmailValidator::new(#field)),
            Validator::Url => quote!(::validator::UrlValidator::new(#field)),
            Validator::IpAddr => quote!(::validator::IpAddrValidator::new(#field)),
            Validator::Length(min, max) => {
                let min = option_literal(min.as_ref());
                let max = option_literal(max.as_ref());
                quote!(::validator::LengthValidator::new(#field, #min, #max))
            }
            Validator::Default => quote!(<#ty as ::validator::Validate>::validator()),
            Validator::Elements(elements) => {
                let element_type = quote!(<#ty as ::validator::HasElements>::Item);
                let elements = elements.create(field, &element_type);
                quote!(::validator::ElementsValidator::new(#elements))
            }
            Validator::Tuple(children) => {
                let children = children.iter().map(|child| child.create(field, ty));
                quote! {
                    (#(#children,)*)
                }
            }
            Validator::Range(range) => {
                quote!(::validator::RangeValidator::new(#field, #range))
            }
            Validator::Ignore => {
                quote!(::validator::IgnoreValidator)
            }
            Validator::Matches(other) => {
                let other = match other {
                    Member::Named(name) => name.to_string(),
                    Member::Unnamed(index) => index.index.to_string()
                };
                quote!(::validator::Matches::new(#field, #other))
            }
            Validator::ParseAs(ty) => {
                quote!(::validator::ParseAs::<#ty>::new(#field))
            }
            Validator::Option(inner) => {
                let ty = quote!(<#ty as ::validator::OptionValue>::Inner);
                let inner = inner.create(field, &ty);
                quote!(::validator::OptionValidator::new(#inner))
            }
        }
    }
    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        match self {
            Validator::NotEmpty => quote!(::validator::NotEmptyValidator),
            Validator::And(left, right) => {
                let left = left.validator_type(ty);
                let right = right.validator_type(ty);
                quote!(::validator::And<#left, #right>)
            }
            Validator::Or(left, right) => {
                let left = left.validator_type(ty);
                let right = right.validator_type(ty);
                quote!(::validator::Or<#left, #right>)
            }
            Validator::Email => quote!(::validator::EmailValidator),
            Validator::Url => quote!(::validator::UrlValidator),
            Validator::IpAddr => quote!(::validator::IpAddrValidator),
            Validator::Length(_, _) => quote!(::validator::LengthValidator),
            Validator::Default => quote!(<#ty as ::validator::Validate>::Validator),
            Validator::Elements(elements) => {
                let element_type = quote!(<#ty as ::validator::HasElements>::Item);
                let elements = elements.validator_type(&element_type);
                quote!(::validator::ElementsValidator<#elements>)
            }
            Validator::Tuple(children) => {
                let children = children.iter().map(|child| child.validator_type(ty));
                quote! {
                    (#(#children,)*)
                }
            }
            Validator::Range(range) => {
                let range = range_type(range, ty);
                quote!(::validator::RangeValidator<#range>)
            }
            Validator::Ignore => {
                quote!(::validator::IgnoreValidator)
            }
            Validator::Matches(_) => {
                quote!(::validator::Matches)
            }
            Validator::ParseAs(ty) => {
                quote!(::validator::ParseAs<#ty>)
            }
            Validator::Option(inner) => {
                let ty = quote!(<#ty as ::validator::OptionValue>::Inner);
                let inner = inner.validator_type(&ty);
                quote!(::validator::OptionValidator<#inner>)
            }
        }
    }
    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        match self {
            Validator::NotEmpty => quote!(::validator::EmptyValueError),
            Validator::And(left, right) => {
                let left = left.error_type(ty);
                let right = right.error_type(ty);
                quote!(::validator::AndError<#left, #right>)
            }
            Validator::Or(left, right) => {
                let left = left.error_type(ty);
                let right = right.error_type(ty);
                quote!((#left, #right))
            }
            Validator::Email => quote!(::validator::InvalidEmailError),
            Validator::Url => quote!(::validator::UrlValidationError),
            Validator::IpAddr => quote!(::std::net::AddrParseError),
            Validator::Length(_, _) => quote!(::validator::InvalidLengthError),
            Validator::Default => {
                quote!(<<#ty as ::validator::Validate>::Validator as ::validator::Validator<#ty>>::Error)
            }
            Validator::Elements(elements) => {
                let element_type = quote!(<#ty as ::validator::HasElements>::Item);
                let elements = elements.error_type(&element_type);
                quote!(::validator::ElementsInvalid<#elements>)
            }
            Validator::Tuple(children) => {
                let error = &format_ident!("TupleError{}", children.len());
                let children = children
                    .iter()
                    .map(|child| child.error_type(ty));
                quote!(
                    validator::#error<#(#children),*>
                )
            }
            Validator::Range(range) => {
                let range = range_type(range, ty);
                quote!(::validator::NotInRangeError<#range>)
            }
            Validator::Ignore => {
                quote!(::core::convert::Infallible)
            }
            Validator::Matches(_) => {
                quote!(::validator::MatchesError)
            }
            Validator::ParseAs(ty) => {
                quote!(::validator::ParseAsError<#ty>)
            }
            Validator::Option(inner) => {
                let ty = quote!(<#ty as ::validator::OptionValue>::Inner);
                inner.error_type(&ty)
            }
        }
    }
}

fn range_type(range: &ExprRange, value: &TokenStream) -> TokenStream {
    match range {
        ExprRange {
            start: Some(_),
            limits: RangeLimits::HalfOpen(_),
            end: Some(_),
            ..
        } => {
            quote!(::std::ops::Range<#value>)
        }
        ExprRange {
            start: Some(_),
            limits: RangeLimits::HalfOpen(_),
            end: None,
            ..
        } => {
            quote!(::std::ops::RangeFrom<#value>)
        }
        ExprRange {
            start: None,
            limits: RangeLimits::HalfOpen(_),
            end: Some(_),
            ..
        } => {
            quote!(::std::ops::RangeTo<#value>)
        }
        ExprRange {
            start: None,
            limits: RangeLimits::HalfOpen(_),
            end: None,
            ..
        } => {
            quote!(::std::ops::RangeFull<#value>)
        }
        ExprRange {
            start: Some(_),
            limits: RangeLimits::Closed(_),
            end: Some(_),
            ..
        } => {
            quote!(::std::ops::RangeInclusive<#value>)
        }
        ExprRange {
            start: None,
            limits: RangeLimits::Closed(_),
            end: Some(_),
            ..
        } => {
            quote!(::std::ops::RangeToInclusive<#value>)
        }
        _ => unreachable!("unknown range type {}", range.to_token_stream()),
    }
}

fn option_literal<T: ToTokens>(opt: Option<T>) -> TokenStream {
    match opt {
        None => quote! { None },
        Some(value) => quote! { Some(#value) },
    }
}
