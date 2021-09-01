use itertools::Itertools;
use pomelo::pomelo;
use std::{convert::{TryFrom, TryInto}, error::Error, fmt::{Debug, Display}, str::{FromStr, ParseBoolError}};
use rand::distributions::{Distribution, Uniform};

#[derive(Clone, Debug)]
pub enum RollExpr {
    Add(Box<RollExpr>, Box<RollExpr>),
    Sub(Box<RollExpr>, Box<RollExpr>),
    Roll {
        count: u8,
        die: u8,
        kh: Option<u8>,
    },
    Const(u8)
}

#[derive(Debug)]
pub struct Roll(i32, Vec<u8>);
impl Roll {
    pub fn value(&self) -> i32 {
        self.0
    }
    pub fn dice<'a>(&'a self) -> impl Iterator<Item=u8> + 'a {
        self.1.iter().copied()
    }
}
impl RollExpr {
    pub fn roll(&self) -> Roll {
        match self {
            RollExpr::Add(l, r) => {
                let Roll(l, mut ls) = l.roll();
                let Roll(r, mut rs) = r.roll();
                ls.append(&mut rs);
                Roll(l + r, ls)
            },
            RollExpr::Sub(l, r) => {
                let Roll(l, mut ls) = l.roll();
                let Roll(r, mut rs) = r.roll();
                ls.append(&mut rs);
                Roll(l - r, ls)

            }
            RollExpr::Roll { count, die, kh } => {
                let mut rng = rand::thread_rng();
                let die = Uniform::from(1..(die+1));
                let mut vals: Vec<_> = die.sample_iter(&mut rng).take(usize::from(*count)).collect();
                vals.sort();
                match kh {
                    None => {}
                    Some(kh) => {
                        vals.truncate(usize::from(*kh));
                    }
                };

                let sum: i32 = vals.iter().map(|x| i32::from(*x)).sum();
                Roll(sum.into(), vals)
            },
            RollExpr::Const(c) => {
                Roll(i32::from(*c), vec![])
            },
        }
    }
}

impl FromStr for RollExpr {
    type Err = Box<ParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s).map_err(Box::new)?;
        let mut p = parser::Parser::new();
        for t in tokens {
            p.parse(t).map_err(|_| Box::new(ParseError::Parse))?;
        }
        Ok(p.end_of_input().map_err(|_| Box::new(ParseError::Parse))?)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Parse,
    Token
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Error for ParseError {
}

fn tokenize(mut s: &str) -> Result<Vec<parser::Token>, ParseError> {
    use parser::Token;

    let mut tokens = vec![];
    let mut iter = Box::new(s.chars());

    loop {
        let num: String = iter.take_while_ref(|c| c.is_ascii_digit()).collect();
        if !num.is_empty() {
            tokens.push(Token::CONST(num.parse().map_err(|_| ParseError::Token)?));
            continue;
        }
        let c = match iter.next() {
            None => return Ok(tokens),
            Some(c) => c,
        };

        match c {
            '+' => tokens.push(Token::PLUS),
            '-' => tokens.push(Token::MINUS),
            'd' => tokens.push(Token::D),
            'k' => match iter.next() {
                Some('h') => tokens.push(Token::KH),
                _ => return Err(ParseError::Token)
            },
            _ => if !c.is_whitespace() {
                return Err(ParseError::Token)
            }
        }
    }
}

pomelo! {
    %include {use super::*;}
    %type CONST u8;
    %type input RollExpr;
    %type expr RollExpr;
    %left PLUS MINUS;


    input ::= expr(E) { E };
    expr ::= expr(L) PLUS expr(R) {RollExpr::Add(Box::new(L), Box::new(R))}
    expr ::= expr(L) MINUS expr(R) {RollExpr::Sub(Box::new(L), Box::new(R))}
    expr ::= CONST(C) D CONST(D) KH CONST(KH) {RollExpr::Roll {
        count: C,
        die: D,
        kh: Some(KH)
    }}
    expr ::= CONST(C) D CONST(D) {RollExpr::Roll {
        count: C,
        die: D,
        kh: None
    }}
    expr ::= CONST(C) {RollExpr::Const(C)}
}