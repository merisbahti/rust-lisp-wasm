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

fn parse_keyword<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(context("keyword", alphanumeric1), |sym_str: &str| {
        Expr::Keyword(sym_str.to_string())
    })(i)
}

fn parse_list<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(
        context("list", delimited(char('('), many0(parse_expr), char(')'))),
        |list| Expr::List(list),
    )(i)
}

fn parse_quote<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(
        context("quote", preceded(tag("'"), parse_list)),
        |exprs| match exprs {
            Expr::List(xs) => Expr::Quote(xs),
            _ => panic!("oops"),
        },
    )(i)
}

fn parse_expr<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    delimited(
        multispace0,
        alt((parse_list, parse_quote, parse_keyword)),
        multispace0,
    )(i)
}

pub fn parse<'a>(input: &'a str) -> Result<Expr, String> {
    parse_expr(input)
        .map_err(|e| {
            let err = format!("{:#?}", e);
            print!("{err}");
            err
        })
        .and_then(|(_, exp)| Ok(exp))
}

#[test]
fn test_parse_alphanumerics() {
    fn kw<'a>(string: &'a str) -> Result<Expr, String> {
        Ok(Expr::Keyword(string.to_string()))
    }
    assert_eq!(parse("true"), kw("true"));
    assert_eq!(parse("false"), kw("false"));
    assert_eq!(parse("1"), kw("1"));
    assert_eq!(parse("5"), kw("5"));

    assert_eq!(parse("  true  "), kw("true"));
    assert_eq!(parse("  false  "), kw("false"));
    assert_eq!(parse("  1  "), kw("1"));
    assert_eq!(parse("  5  "), kw("5"));
}

#[test]
fn test_parse_alphanumerics_fails() {
    assert!(parse("--::--").is_err());
    assert!(parse("'").is_err());
    assert!(parse("'dsa").is_err());
}

#[test]
fn test_parse_lists() {
    fn ok_list<'a>(strings: Vec<&'a str>) -> Result<Expr, String> {
        Ok(Expr::List(
            strings
                .iter()
                .map(|x| Expr::Keyword(x.to_string()))
                .collect(),
        ))
    }
    assert_eq!(parse("(a)"), ok_list(vec!("a")));
    assert_eq!(parse("(123)"), ok_list(vec!("123")));
    assert_eq!(
        parse("(     a         1       b          2      )"),
        ok_list(vec!("a", "1", "b", "2"))
    );
    assert_eq!(
        parse("(     a         (wat    woo    wii)       b          2      )"),
        Ok(Expr::List(vec![
            Expr::Keyword("a".to_string()),
            Expr::List(vec!(
                Expr::Keyword("wat".to_string()),
                Expr::Keyword("woo".to_string()),
                Expr::Keyword("wii".to_string())
            )),
            Expr::Keyword("b".to_string()),
            Expr::Keyword("2".to_string())
        ]))
    );
    //assert_eq!(parse("(123)"), Ok(Expr::Keyword("5".to_string())));
}
