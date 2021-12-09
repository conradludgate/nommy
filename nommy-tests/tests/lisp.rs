use std::{convert::TryFrom, str::FromStr};

use nommy::{text::*, Parse, parse};

type Ident = WhileNot1<" \t\r\n)">;

#[derive(Debug, PartialEq, Parse)]
#[nommy(ignore = WhiteSpace)]
#[nommy(parse_type = char)]
enum Exp {
    #[nommy(prefix = Tag<"(">, suffix = Tag<")">)]
    SExp(Vec<Exp>),
    Number(#[nommy(parser = Number)] f64),
    Symbol(#[nommy(parser = Ident)] String),
}

type Integer = AnyOf1<"0123456789">;
#[derive(Debug, PartialEq, Parse)]
struct Number {
    #[nommy(parser = Integer)]
    integer: String,
    decimal: Option<Decimal>,
}

#[derive(Debug, PartialEq, Parse)]
#[nommy(prefix = Tag<".">)]
struct Decimal(#[nommy(parser = Integer)] String);

impl ToString for Number {
    fn to_string(&self) -> String {
        match &self.decimal {
            Some(d) => format!("{}.{}", self.integer, d.0),
            None => self.integer.clone(),
        }
    }
}

impl TryFrom<Number> for f64 {
    type Error = <f64 as FromStr>::Err;

    fn try_from(number: Number) -> Result<Self, Self::Error> {
        f64::from_str(&number.to_string())
    }
}

impl Exp {
    fn eval(&self) -> f64 {
        match &self {
            Exp::Number(n) => *n,
            Exp::SExp(args) => {
                if let Some((Exp::Symbol(symbol), rest)) = args.split_first() {
                    match symbol.as_str() {
                        "+" => rest.iter().map(|e| e.eval()).sum(),
                        "*" => rest.iter().map(|e| e.eval()).product(),
                        symbol => panic!("unknown symbol {}", symbol),
                    }
                } else {
                    panic!("unknown s-exp")
                }
            }
            Exp::Symbol(symbol) => panic!("cannot eval symbol {}", symbol),
        }
    }
}

fn main() -> nommy::eyre::Result<()> {
    let input = "(+ 4 (* 3 5.5))".chars();
    let exp: Exp = parse(input)?;
    println!("{:?} == {}", exp, exp.eval());

    Ok(())
}
