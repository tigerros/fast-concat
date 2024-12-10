use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct FastConcatParser {
    pub args: Vec<Expr>,
}

impl Parse for FastConcatParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        Ok(Self { args: args.into_iter().collect() })
    }
}
