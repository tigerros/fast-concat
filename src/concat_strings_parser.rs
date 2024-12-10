use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(strict);
}

pub struct ConcatStringsParser {
    pub args: Vec<Expr>,
    pub strict: bool,
}

impl Parse for ConcatStringsParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let strict: Option<kw::strict> = input.parse()?;

        if strict.is_some() {
            input.parse::<Token![:]>()?;
        }

        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        Ok(Self { args: args.into_iter().collect(), strict: strict.is_some() })
    }
}
