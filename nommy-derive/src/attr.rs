use proc_macro2::{Delimiter, TokenStream, TokenTree};

#[derive(Default, Debug, Clone)]
pub struct GlobalAttr {
    pub ignore_whitespace: Option<IgnoreWS>,
    pub prefix: Option<syn::Type>,
    pub suffix: Option<syn::Type>,
}

impl GlobalAttr {
    pub fn parse_attr(&mut self, tokens: TokenStream) {
        for tt in tokens.into_iter() {
            let inner = match tt {
                TokenTree::Group(g) => {
                    if g.delimiter() == Delimiter::Parenthesis {
                        g.stream()
                    } else {
                        panic!("unexpected token")
                    }
                }
                _ => panic!("unexpected token"),
            };

            self.parse_args(inner);
        }
    }

    pub fn parse_args(&mut self, tokens: TokenStream) {
        let mut stream = TokenStream::new();
        for tt in tokens {
            match tt {
                TokenTree::Punct(p) => {
                    if p.as_char() == ',' {
                        let mut tmp = TokenStream::new();
                        std::mem::swap(&mut stream, &mut tmp);
                        self.parse_arg(tmp)
                    } else {
                        stream.extend(vec![TokenTree::Punct(p)])
                    }
                }
                _ => stream.extend(vec![tt]),
            }
        }
        self.parse_arg(stream)
    }

    pub fn parse_arg(&mut self, tokens: TokenStream) {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            _ => panic!("expected ident")
        };

        match ident.to_string().as_ref() {
            "ignore_whitespace" => self.parse_ignore_ws(tokens),
            "prefix" => self.parse_prefix(tokens),
            "suffix" => self.parse_suffix(tokens),
            s => panic!("unknown parameter {}", s),
        }
    }

    pub fn parse_ignore_ws(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            None => {
                self.ignore_whitespace = Some(IgnoreWS::All);
                return
            }
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        match tokens.next() {
            Some(TokenTree::Literal(s)) => {
                match s.to_string().as_ref() {
                    "\"spaces\"" => self.ignore_whitespace = Some(IgnoreWS::Spaces),
                    "\"all\"" => self.ignore_whitespace = Some(IgnoreWS::All),
                    _ => panic!("unsupported term. can be \"spaces\" or \"all\"")
                }
            }
            _ => panic!("unsupported term. can be \"spaces\" or \"all\"")
        }
    }

    pub fn parse_prefix(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        let mut stream = TokenStream::new();
        stream.extend(tokens);

        let ty: syn::Type = syn::parse2(stream).expect("could not parse type");
        self.prefix = Some(ty)
    }

    pub fn parse_suffix(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        let mut stream = TokenStream::new();
        stream.extend(tokens);

        let ty: syn::Type = syn::parse2(stream).expect("could not parse type");
        self.suffix = Some(ty)
    }
}

#[derive(Debug, Clone)]
pub enum IgnoreWS {
    Spaces,
    All,
}

#[derive(Default, Debug, Clone)]
pub struct FieldAttr {
    pub prefix: Option<syn::Type>,
    pub suffix: Option<syn::Type>,
    pub parser: Option<syn::Type>,
    pub vec: VecFieldAttr,
}
#[derive(Default, Debug, Clone)]
pub struct VecFieldAttr {
    pub count: Option<syn::Expr>,
    pub min: Option<syn::Expr>,
    pub max: Option<syn::Expr>,
}

impl FieldAttr {
    pub fn parse_attr(&mut self, tokens: TokenStream) {
        for tt in tokens.into_iter() {
            let inner = match tt {
                TokenTree::Group(g) => {
                    if g.delimiter() == Delimiter::Parenthesis {
                        g.stream()
                    } else {
                        panic!("unexpected token")
                    }
                }
                _ => panic!("unexpected token"),
            };

            self.parse_args(inner);
        }
    }

    pub fn parse_args(&mut self, tokens: TokenStream) {
        let mut stream = TokenStream::new();
        for tt in tokens {
            match tt {
                TokenTree::Punct(p) => {
                    if p.as_char() == ',' {
                        let mut tmp = TokenStream::new();
                        std::mem::swap(&mut stream, &mut tmp);
                        self.parse_arg(tmp)
                    } else {
                        stream.extend(vec![TokenTree::Punct(p)])
                    }
                }
                _ => stream.extend(vec![tt]),
            }
        }
        self.parse_arg(stream)
    }

    pub fn parse_arg(&mut self, tokens: TokenStream) {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            _ => panic!("expected ident")
        };

        match ident.to_string().as_ref() {
            "prefix" => self.parse_prefix(tokens),
            "suffix" => self.parse_suffix(tokens),
            "parser" => self.parse_parser(tokens),
            s => panic!("unknown parameter {}", s),
        }
    }

    pub fn parse_prefix(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        let mut stream = TokenStream::new();
        stream.extend(tokens);

        let ty: syn::Type = syn::parse2(stream).expect("could not parse type");
        self.prefix = Some(ty)
    }

    pub fn parse_suffix(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        let mut stream = TokenStream::new();
        stream.extend(tokens);

        let ty: syn::Type = syn::parse2(stream).expect("could not parse type");
        self.suffix = Some(ty)
    }

    pub fn parse_parser(&mut self, mut tokens: proc_macro2::token_stream::IntoIter) {
        match tokens.next() {
            Some(TokenTree::Punct(p)) => {
                if p.as_char() != '=' {
                    panic!("expected an '=' to follow")
                }
            }
            _ => panic!("expected an '=' to follow"),
        }

        let mut stream = TokenStream::new();
        stream.extend(tokens);

        let ty: syn::Type = syn::parse2(stream).expect("could not parse type");
        self.parser = Some(ty)
    }
}
