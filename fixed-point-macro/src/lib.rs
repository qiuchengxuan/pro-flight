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
    decimal_length: syn::LitInt,
}

impl syn::parse::Parse for FixedPointMacroInput {
    fn parse(tokens: syn::parse::ParseStream) -> syn::Result<Self> {
        let float: syn::LitFloat = tokens.parse()?;
        let _comma: Token![,] = tokens.parse()?;
        let decimal_length: syn::LitInt = tokens.parse()?;
        Ok(FixedPointMacroInput { float, decimal_length })
    }
}

#[proc_macro]
pub fn fixed_point(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as FixedPointMacroInput);
    let decimal_length: usize = input.decimal_length.base10_digits().parse().unwrap();
    let num_string = input.float.to_string();
    let exp = decimal_length - num_string.rsplitn(2, ".").next().unwrap().replace("_", "").len();
    let mul = 10_isize.pow(exp as u32);
    let number = num_string.replace(&['.', '_'][..], "").parse::<isize>().unwrap() * mul;
    let number = Literal::isize_unsuffixed(number);
    let decimal_length = Literal::usize_unsuffixed(decimal_length);
    let type_ = Ident::new(input.decimal_length.suffix(), Span::call_site());
    quote!(FixedPoint::<#type_, #decimal_length>(#number)).into()
}
