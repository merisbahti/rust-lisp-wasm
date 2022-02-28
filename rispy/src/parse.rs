use nom::number::complete::double;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, multispace0},
    combinator::map,
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

use crate::expr::Expr;

fn parse_number(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("number", double), |nr| Expr::Num(nr))(i)
}

fn parse_keyword(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("keyword", alphanumeric1), |sym_str: &str| {
        Expr::Keyword(sym_str.to_string())
    })(i)
}

fn parse_list(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        context("list", delimited(char('('), many0(parse_expr), char(')'))),
        |list| Expr::List(list),
    )(i)
}

fn parse_quote(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        context("quote", preceded(tag("'"), parse_list)),
        |exprs| match exprs {
            Expr::List(xs) => Expr::Quote(xs),
            _ => panic!("oops"),
        },
    )(i)
}

fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    delimited(
        multispace0,
        alt((parse_list, parse_number, parse_quote, parse_keyword)),
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
    assert_eq!(parse("false"), kw("false"));
    assert_eq!(parse("1"), nr(1.));
    assert_eq!(parse("5"), nr(5.));

    assert_eq!(parse("  true  "), kw("true"));
    assert_eq!(parse("  false  "), kw("false"));
    assert_eq!(parse("  1  "), nr(1.));
    assert_eq!(parse("  5  "), nr(5.));
}

#[test]
fn test_parse_alphanumerics_fails() {
    assert!(parse("--::--").is_err());
    assert!(parse("'").is_err());
    assert!(parse("'dsa").is_err());
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
