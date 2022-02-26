#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};

struct FixedPointMacroInput {
    float: syn::LitFloat,
    decimal_length: Option<syn::LitInt>,
}

impl syn::parse::Parse for FixedPointMacroInput {
    fn parse(tokens: syn::parse::ParseStream) -> syn::Result<Self> {
        let float: syn::LitFloat = tokens.parse()?;
        let decimal_length = match tokens.parse::<Token![,]>() {
            Ok(_) => Some(tokens.parse::<syn::LitInt>()?),
            Err(_) => None,
        };
        Ok(FixedPointMacroInput { float, decimal_length })
    }
}

#[proc_macro]
pub fn fixed(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as FixedPointMacroInput);
    let num_string = input.float.base10_digits();
    let mut number = num_string.replace('.', "").parse::<isize>().unwrap();
    let decimal = num_string.rsplit('.').next().unwrap();
    let decimal_len = decimal.chars().filter(|&c| c != '_').count() as u8;
    if input.float.suffix() == "" && input.decimal_length.is_none() {
        let number = Literal::isize_unsuffixed(number);
        let decimal_len = Literal::u8_unsuffixed(decimal_len);
        return quote!(FixedPoint::new(#number, #decimal_len)).into();
    }
    let decimal_length: u8 =
        input.decimal_length.map(|x| x.base10_digits().parse().unwrap()).unwrap_or(decimal_len);
    if decimal_length < decimal_len {
        let error = |s| syn::Error::new(Span::call_site(), s).to_compile_error();
        return error("Insufficient precision").into();
    }
    number = number * 10_isize.pow((decimal_length - decimal_len) as u32);
    let unsuffixed = Literal::isize_unsuffixed(number);
    if input.float.suffix() == "" {
        return quote!(FixedPoint(#unsuffixed)).into();
    }
    let type_ = Ident::new(input.float.suffix(), Span::call_site());
    quote!(FixedPoint::<#type_, #decimal_length>(#unsuffixed)).into()
}
