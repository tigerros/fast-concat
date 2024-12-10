use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(strict);
}

pub struct ConcatPathsParser {
    pub args: Vec<Expr>,
}

impl Parse for ConcatPathsParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        Ok(Self { args: args.into_iter().collect(), })
    }
}
