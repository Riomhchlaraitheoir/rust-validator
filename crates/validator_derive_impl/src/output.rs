use crate::{
    And, Default, Elements, Email, EnumVariant, Field, Ignore, Input, InputData, IpAddr, Length,
    Matches, NotEmpty, Optional, Or, ParseAs, Range, StructFields, Tuple, Url,
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{ExprRange, Index, Member, RangeLimits};

impl StructFields {
    fn validator_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(Field::define_validator_field);
        if self.named_fields {
            quote! {{#(#fields),*}}
        } else {
            quote!((#(#fields),*))
        }
    }

    fn error_definition(&self) -> TokenStream {
        let fields = self.fields.iter().map(Field::error_field);
        if self.named_fields {
            quote! {{
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
                    let types = fields
                        .fields
                        .iter()
                        .map(|f| f.validator.error_type(&f.ty.to_token_stream()));
                    let field_vis = fields.fields.iter().map(|f| &f.vis);
                    quote! {
                        #[derive(Debug, PartialEq, Clone, Default)]
                        #vis struct #error_type {
                            #(#field_vis #members: Option<#types>,)*
                        }
                    }
                } else {
                    let types = fields
                        .fields
                        .iter()
                        .map(|f| f.validator.error_type(&f.ty.to_token_stream()));
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

                    #[automatically_derived]
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

                    #[automatically_derived]
                    impl ::validator::Validator<#derived_type> for #validator_type {
                        type Error = #error_type;
                        #[allow(non_shorthand_field_patterns)]
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
                    #[automatically_derived]
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
                    #[automatically_derived]
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
        let field = self.pattern_name();
        self.validator.validate(&self.name, &field)
    }

    fn create_validator(&self) -> TokenStream {
        self.validator
            .create(&self.label, &self.ty.to_token_stream())
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
            quote! {{#(#validator_fields),*}}
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

pub trait ValidatorOutput {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream;
    fn validator_type(&self, ty: &TokenStream) -> TokenStream;
    fn error_type(&self, ty: &TokenStream) -> TokenStream;

    fn validate(&self, name: &Member, field: &Ident) -> TokenStream {
        quote!(
            {
                match validator.#name.validate(#field) {
                    Ok(()) => None,
                    Err(error) => {
                        _valid = false;
                        Some(error)
                    }
                }
            }
        )
    }
}

impl ValidatorOutput for NotEmpty {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::not_empty::NotEmptyValidator::new(#field))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::not_empty::NotEmptyValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::not_empty::EmptyValueError)
    }
}
impl ValidatorOutput for And {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.create(field, ty));
        quote!(::validator::and::And::new(#(#children,)*))
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.validator_type(ty));
        quote!(::validator::and::And<#(#children,)*>)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.error_type(ty));
        quote!(::validator::and::AndError<#(#children,)*>)
    }
}
impl ValidatorOutput for Or {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.create(field, ty));
        quote!(::validator::or::Or::new(#(#children,)*))
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.validator_type(ty));
        quote!(::validator::or::Or<#(#children,)*>)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.error_type(ty));
        quote!(::validator::or::OrError<#(#children,)*>)
    }
}
impl ValidatorOutput for Email {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::email::EmailValidator::new(#field))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::email::EmailValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::email::InvalidEmailError)
    }
}
impl ValidatorOutput for Url {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::url::UrlValidator::new(#field))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::url::UrlValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::url::UrlValidationError)
    }
}
impl ValidatorOutput for IpAddr {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::ip::IpAddrValidator::new(#field))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::ip::IpAddrValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::std::net::AddrParseError)
    }
}
impl ValidatorOutput for Length {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        let range = &self.0;
        quote!(::validator::length::LengthValidator::new(#field, #range))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::length::LengthValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::length::InvalidLengthError)
    }
}
impl ValidatorOutput for Elements {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        let element_type = quote!(<#ty as ::validator::elements::HasElements>::Item);
        let elements = self.0.create(field, &element_type);
        quote!(::validator::elements::ElementsValidator::new(#elements))
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let element_type = quote!(<#ty as ::validator::elements::HasElements>::Item);
        let elements = self.0.validator_type(&element_type);
        quote!(::validator::elements::ElementsValidator<#elements>)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let element_type = quote!(<#ty as ::validator::elements::HasElements>::Item);
        let elements = self.0.error_type(&element_type);
        quote!(::validator::elements::ElementsInvalid<#elements>)
    }
}
impl ValidatorOutput for Default {
    fn create(&self, _field: &str, ty: &TokenStream) -> TokenStream {
        quote!(<#ty as ::validator::Validate>::validator())
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        quote!(<#ty as ::validator::Validate>::Validator)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        quote!(<<#ty as ::validator::Validate>::Validator as ::validator::Validator<#ty>>::Error)
    }
}
impl ValidatorOutput for Ignore {
    fn create(&self, _field: &str, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::IgnoreValidator)
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::IgnoreValidator)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::core::convert::Infallible)
    }
}
impl ValidatorOutput for Tuple {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.create(field, ty));
        quote! {
            (#(#children,)*)
        }
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let children = self.0.iter().map(|child| child.validator_type(ty));
        quote! {
            (#(#children,)*)
        }
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let error = &format_ident!("TupleError{}", self.0.len());
        let children = self.0.iter().map(|child| child.error_type(ty));
        quote!(
            validator::#error<#(#children),*>
        )
    }
}
impl ValidatorOutput for Range {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        let range = &self.0;
        quote!(::validator::range::RangeValidator::new(#field, #range))
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let range = range_type(&self.0, ty);
        quote!(::validator::range::RangeValidator<#range>)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let range = range_type(&self.0, ty);
        quote!(::validator::range::NotInRangeError<#range>)
    }
}
impl ValidatorOutput for Matches {
    fn create(&self, field: &str, _ty: &TokenStream) -> TokenStream {
        let other = match &self.0 {
            Member::Named(name) => name.to_string(),
            Member::Unnamed(index) => index.index.to_string(),
        };
        quote!(::validator::matches::Matches::new(#field, #other))
    }

    fn validator_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::matches::Matches)
    }

    fn error_type(&self, _ty: &TokenStream) -> TokenStream {
        quote!(::validator::matches::MatchesError)
    }

    fn validate(&self, name: &Member, field: &Ident) -> TokenStream {
        let other = match &self.0 {
            Member::Named(name) => name.clone(),
            Member::Unnamed(index) => format_ident!("value{}", index.index),
        };
        quote!(
            {
                match validator.#name.validate(&(#field, #other)) {
                    Ok(()) => None,
                    Err(error) => {
                        _valid = false;
                        Some(error)
                    }
                }
            }
        )
    }
}
impl ValidatorOutput for ParseAs {
    fn create(&self, field: &str, _: &TokenStream) -> TokenStream {
        let ty = &self.0;
        quote!(::validator::parse_as::ParseAs::<#ty>::new(#field))
    }

    fn validator_type(&self, _: &TokenStream) -> TokenStream {
        let ty = &self.0;
        quote!(::validator::parse_as::ParseAs<#ty>)
    }

    fn error_type(&self, _: &TokenStream) -> TokenStream {
        let ty = &self.0;
        quote!(::validator::parse_as::ParseAsError<#ty>)
    }
}
impl ValidatorOutput for Optional {
    fn create(&self, field: &str, ty: &TokenStream) -> TokenStream {
        let ty = quote!(<#ty as ::validator::option::OptionValue>::Inner);
        let inner = self.0.create(field, &ty);
        quote!(::validator::option::OptionValidator::new(#inner))
    }

    fn validator_type(&self, ty: &TokenStream) -> TokenStream {
        let ty = quote!(<#ty as ::validator::option::OptionValue>::Inner);
        let inner = self.0.validator_type(&ty);
        quote!(::validator::option::OptionValidator<#inner>)
    }

    fn error_type(&self, ty: &TokenStream) -> TokenStream {
        let ty = quote!(<#ty as ::validator::option::OptionValue>::Inner);
        self.0.error_type(&ty)
    }
}
