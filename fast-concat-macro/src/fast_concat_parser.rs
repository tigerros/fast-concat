use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct Item {
    pub is_const: bool,
    pub expr: Expr,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_const = input.parse::<Option<Token![const]>>()?.is_some();
        let expr = input.parse::<Expr>()?;

        Ok(Self { is_const, expr })
    }
}

pub struct FastConcatParser {
    pub items: Vec<Item>,
}

impl Parse for FastConcatParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::<Item, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            items: items.into_iter().collect(),
        })
    }
}
