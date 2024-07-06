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

pub fn make_pair_from_vec(v: Vec<Expr>) -> Expr {
    match v.split_first() {
        Some((head, tail)) => Expr::Pair(
            Box::new(head.clone()),
            Box::new(make_pair_from_vec(tail.to_vec()).clone()),
        ),
        None => Expr::Nil,
    }
}

fn parse_list(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        context("list", delimited(char('('), many0(parse_expr), char(')'))),
        make_pair_from_vec,
    )(i)
}

fn parse_quote(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("quote", preceded(tag("'"), parse_expr)), |exprs| {
        Expr::Quote(Box::new(exprs))
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
        Expr::Quote(box Expr::Nil) => true,
        _ => false,
    });

    let res2 = parse("'(a b)").map(|x| x.get(0).cloned()).unwrap().unwrap();

    assert!(match res2 {
        Expr::Quote(box rc) => match rc {
            Expr::Pair(
                box Expr::Keyword(a),
                box Expr::Pair(box Expr::Keyword(b), box Expr::Nil),
            ) => a == "a".to_string() && b == "b".to_string(),
            _ => false,
        },
        _ => false,
    });

    let res3 = parse("'a").map(|x| x.get(0).cloned()).unwrap().unwrap();

    assert!(match res3 {
        Expr::Quote(box rc) => match rc {
            Expr::Keyword(kw) => kw == "a",
            _ => panic!("found: {:?}", rc),
        },
        _ => panic!("found: {:?}", res3),
    });
}

#[test]
fn test_parse_lists() {
    fn ok_list(strings: Vec<&str>) -> Result<Vec<Expr>, String> {
        let stuff: Vec<Expr> = strings
            .iter()
            .map(|x| Expr::Keyword(x.to_string()))
            .collect();
        Ok(vec![make_pair_from_vec(stuff)])
    }
    assert_eq!(parse("(a)"), ok_list(vec!(("a"))));
    assert_eq!(
        parse("(123)"),
        Ok(vec![Expr::Pair(
            Box::new(Expr::Num(123.)),
            Box::new(Expr::Nil)
        )])
    );
    assert_eq!(
        parse("(     a         1       b          2      )"),
        Ok(vec![make_pair_from_vec(vec![
            Expr::Keyword("a".to_string()),
            Expr::Num(1.),
            Expr::Keyword("b".to_string()),
            Expr::Num(2.)
        ])])
    );
    assert_eq!(
        parse("(     a         (wat    woo    wii)       b          2      )"),
        Ok(vec![make_pair_from_vec(vec![
            Expr::Keyword("a".to_string()),
            make_pair_from_vec(vec!(
                Expr::Keyword("wat".to_string()),
                Expr::Keyword("woo".to_string()),
                Expr::Keyword("wii".to_string())
            )),
            Expr::Keyword("b".to_string()),
            Expr::Num(2.)
        ])])
    );
}
