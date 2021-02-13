use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};

#[derive(Default)]
pub struct GlobalAttr {
    pub ignore: Vec<syn::Type>,
    pub debug: bool,
    pub prefix: Option<syn::Type>,
    pub suffix: Option<syn::Type>,
    pub parse_type: Option<syn::Type>,
}

fn parse_type(
    span: Span,
    mut tokens: proc_macro2::token_stream::IntoIter,
) -> syn::Result<syn::Type> {
    match tokens.next() {
        Some(TokenTree::Punct(p)) => {
            if p.as_char() != '=' {
                return Err(syn::Error::new_spanned(p, "expected an '=' to follow"));
            }
        }
        Some(t) => return Err(syn::Error::new_spanned(t, "expected an '=' to follow")),
        None => return Err(syn::Error::new(span, "expected an '=' to follow")),
    }

    let mut stream = TokenStream::new();
    stream.extend(tokens);

    syn::parse2(stream)
}

impl GlobalAttr {
    pub fn extend_with(mut self, extend: &Self) -> Self {
        self.ignore.extend_from_slice(&extend.ignore);
        self.parse_type = extend.parse_type.clone();
        self
    }

    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut output = GlobalAttr::default();
        for attr in attrs {
            if attr.path.is_ident("nommy") {
                output.parse_attr(attr.tokens)?;
            }
        }
        Ok(output)
    }

    pub fn parse_attr(&mut self, tokens: TokenStream) -> syn::Result<()> {
        for tt in tokens.into_iter() {
            let (inner, span) = match tt {
                TokenTree::Group(g) => {
                    if g.delimiter() == Delimiter::Parenthesis {
                        (g.stream(), g.span())
                    } else {
                        return Err(syn::Error::new_spanned(
                            g,
                            "expected parenthesied attribute arguments",
                        ));
                    }
                }
                t => {
                    return Err(syn::Error::new_spanned(
                        t,
                        "expected parenthesied attribute arguments",
                    ))
                }
            };

            self.parse_args(span, inner)?;
        }
        Ok(())
    }

    pub fn parse_args(&mut self, mut span: Span, tokens: TokenStream) -> syn::Result<()> {
        let mut stream = TokenStream::new();
        for tt in tokens {
            match tt {
                TokenTree::Punct(p) => {
                    if p.as_char() == ',' {
                        let mut tmp = TokenStream::new();
                        std::mem::swap(&mut stream, &mut tmp);
                        self.parse_arg(span, tmp)?;
                        span = p.span();
                    } else {
                        stream.extend(vec![TokenTree::Punct(p)])
                    }
                }
                _ => stream.extend(vec![tt]),
            }
        }
        self.parse_arg(span, stream)
    }

    pub fn parse_arg(&mut self, span: Span, tokens: TokenStream) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(t) => return Err(syn::Error::new_spanned(t, "expected ident")),
            None => return Err(syn::Error::new(span, "expected ident to follow")),
        };

        match ident.to_string().as_ref() {
            "ignore" => self.ignore.push(parse_type(ident.span(), tokens)?),
            "prefix" => self.prefix = Some(parse_type(ident.span(), tokens)?),
            "suffix" => self.suffix = Some(parse_type(ident.span(), tokens)?),
            "parse_type" => self.parse_type = Some(parse_type(ident.span(), tokens)?),
            "debug" => self.debug = true,
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct FieldAttr {
    pub prefix: Option<syn::Type>,
    pub suffix: Option<syn::Type>,
    pub parser: Option<syn::Type>,
    pub vec: VecFieldAttr,
}
#[derive(Default)]
pub struct VecFieldAttr {
    pub count: Option<syn::Expr>,
    pub min: Option<syn::Expr>,
    pub max: Option<syn::Expr>,
    pub parser: Option<syn::Type>,
    pub seperated_by: Option<syn::Type>,
    pub trailing: Option<bool>,
}

impl VecFieldAttr {
    pub fn is_some(&self) -> bool {
        self.count.is_some() || self.min.is_some() || self.max.is_some() || self.parser.is_some()
    }
}

impl FieldAttr {
    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut output = FieldAttr::default();
        for attr in attrs {
            if attr.path.is_ident("nommy") {
                output.parse_attr(attr.tokens)?;
            }
        }
        Ok(output)
    }

    pub fn parse_attr(&mut self, tokens: TokenStream) -> syn::Result<()> {
        for tt in tokens.into_iter() {
            let (inner, span) = match tt {
                TokenTree::Group(g) => {
                    if g.delimiter() == Delimiter::Parenthesis {
                        (g.stream(), g.span())
                    } else {
                        return Err(syn::Error::new_spanned(
                            g,
                            "expected parenthesied attribute arguments",
                        ));
                    }
                }
                t => {
                    return Err(syn::Error::new_spanned(
                        t,
                        "expected parenthesied attribute arguments",
                    ))
                }
            };

            self.parse_args(span, inner)?;
        }
        Ok(())
    }

    pub fn parse_args(&mut self, mut span: Span, tokens: TokenStream) -> syn::Result<()> {
        let mut stream = TokenStream::new();
        for tt in tokens {
            match tt {
                TokenTree::Punct(p) => {
                    if p.as_char() == ',' {
                        let mut tmp = TokenStream::new();
                        std::mem::swap(&mut stream, &mut tmp);
                        self.parse_arg(span, tmp)?;
                        span = p.span();
                    } else {
                        stream.extend(vec![TokenTree::Punct(p)])
                    }
                }
                _ => stream.extend(vec![tt]),
            }
        }
        self.parse_arg(span, stream)
    }

    pub fn parse_arg(&mut self, span: Span, tokens: TokenStream) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(t) => return Err(syn::Error::new_spanned(t, "expected ident")),
            None => return Err(syn::Error::new(span, "expected ident to follow")),
        };

        match ident.to_string().as_ref() {
            "prefix" => self.prefix = Some(parse_type(ident.span(), tokens)?),
            "suffix" => self.suffix = Some(parse_type(ident.span(), tokens)?),
            "parser" => self.parser = Some(parse_type(ident.span(), tokens)?),
            "inner_parser" => self.vec.parser = Some(parse_type(ident.span(), tokens)?),
            "seperated_by" => self.vec.seperated_by = Some(parse_type(ident.span(), tokens)?),
            "trailing" => self.parse_trailing(tokens)?,
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        }
        Ok(())
    }

    pub fn parse_trailing(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) -> syn::Result<()> {
        let span = match tokens.next() {
            None => return Ok(()),
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    return Err(syn::Error::new(p.span(), "expected '=' to follow"))
                }
                p.span()
            }
            Some(other) => return Err(syn::Error::new(other.span(), "expected '=' to follow"))
        };

        match tokens.next() {
            Some(TokenTree::Literal(lit)) => {
                match lit.to_string().as_str() {
                    "\"yes\"" => self.vec.trailing = Some(true),
                    "\"no\"" => self.vec.trailing = None,
                    "\"maybe\"" => self.vec.trailing = Some(false),
                    _ => return Err(syn::Error::new(lit.span(), "expected \"yes\", \"no\" or \"maybe\" to follow")),
                }
            }
            Some(other) => return Err(syn::Error::new(other.span(), "expected \"yes\", \"no\" or \"maybe\" to follow")),
            None => return Err(syn::Error::new(span, "expected \"yes\", \"no\" or \"maybe\" to follow")),
        }

        match tokens.next() {
            Some(other) => return Err(syn::Error::new(other.span(), "expected no more tokens")),
            None => Ok(())
        }

    }
}
