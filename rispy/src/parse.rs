use std::rc::Rc;

use nom::bytes::complete::is_not;
use nom::multi::many1;
use nom::number::complete::double;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

use crate::expr::Expr;

fn parse_number(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("number", double), Expr::Num)(i)
}

fn parse_keyword(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("keyword", many1(is_not("\n\r )("))), |sym_str| {
        Expr::Keyword(sym_str.into_iter().collect())
    })(i)
}

fn parse_list(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        context("list", delimited(char('('), many0(parse_expr), char(')'))),
        Expr::List,
    )(i)
}

fn parse_quote(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("quote", preceded(tag("'"), parse_expr)), |exprs| {
        Expr::Quote(vec![exprs])
    })(i)
}

fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    delimited(
        multispace0,
        alt((parse_quote, parse_list, parse_number, parse_keyword)),
        multispace0,
    )(i)
}

pub fn parse(i: &str) -> Result<Vec<Expr>, String> {
    many0(delimited(multispace0, parse_expr, multispace0))(i)
        .map_err(|e| format!("{e:?}"))
        .and_then(|(remaining, exp)| match remaining {
            "" => Ok(exp),
            remainder => Err(format!("Unexpected end of input: {remainder}")),
        })
}

#[test]
fn test_parse_alphanumerics() {
    fn kw<'a>(string: &'a str) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Keyword(string.to_string())])
    }
    fn nr<'a>(nr: f64) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Num(nr)])
    }

    assert_eq!(parse("true"), kw("true"));
    assert_eq!(parse("+"), kw("+"));
    assert_eq!(parse("'"), kw("'"));
    assert_eq!(parse("false"), kw("false"));
    assert_eq!(parse("1"), nr(1.));
    assert_eq!(parse("5"), nr(5.));

    assert_eq!(parse("  true  "), kw("true"));
    assert_eq!(parse("  false  "), kw("false"));
    assert_eq!(parse("  1  "), nr(1.));
    assert_eq!(parse("  5  "), nr(5.));
}

#[test]
fn test_parse_quote() {
    let res = parse("'()");
    assert_eq!(res.as_ref().map(|x| x.len()), Ok(1));

    let borrowed = res.map(|x| x.get(0).cloned()).unwrap().unwrap();
    assert!(match borrowed {
        Expr::Quote(rc) => match rc.first().unwrap() {
            &Expr::List(ref v) if v.is_empty() => true,
            _ => false,
        },
        _ => false,
    });

    let res2 = parse("'(a b)").map(|x| x.get(0).cloned()).unwrap().unwrap();

    assert!(match res2 {
        Expr::Quote(rc) => match rc.first().unwrap() {
            &Expr::List(ref v)
                if v.get(0).unwrap().clone() == Expr::Keyword("a".to_string())
                    && v.get(1).unwrap().clone() == Expr::Keyword("b".to_string()) =>
                true,
            _ => false,
        },
        _ => false,
    });

    let res3 = parse("'a").map(|x| x.get(0).cloned()).unwrap().unwrap();

    assert!(match res3 {
        Expr::Quote(rc) => match rc.first().unwrap() {
            &Expr::Keyword(ref kw) => kw == "a",
            _ => panic!("found: {:?}", rc),
        },
        _ => panic!("found: {:?}", res3),
    });
}

#[test]
fn test_parse_lists() {
    fn ok_list(strings: Vec<&str>) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::List(
            strings
                .iter()
                .map(|x| Expr::Keyword(x.to_string()))
                .collect(),
        )])
    }
    assert_eq!(parse("(a)"), ok_list(vec!("a")));
    assert_eq!(parse("(123)"), Ok(vec![Expr::List(vec![Expr::Num(123.)])]));
    assert_eq!(
        parse("(     a         1       b          2      )"),
        Ok(vec![Expr::List(vec![
            Expr::Keyword("a".to_string()),
            Expr::Num(1.),
            Expr::Keyword("b".to_string()),
            Expr::Num(2.)
        ])])
    );
    assert_eq!(
        parse("(     a         (wat    woo    wii)       b          2      )"),
        Ok(vec![Expr::List(vec![
            Expr::Keyword("a".to_string()),
            Expr::List(vec!(
                Expr::Keyword("wat".to_string()),
                Expr::Keyword("woo".to_string()),
                Expr::Keyword("wii".to_string())
            )),
            Expr::Keyword("b".to_string()),
            Expr::Num(2.)
        ])])
    );
}
