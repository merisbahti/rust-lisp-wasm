use crate::expr::Expr;
use nom::bytes::complete::is_not;
use nom::character::complete::multispace1;
use nom::combinator::value;
use nom::multi::many1;
use nom::number::complete::double;
use nom::sequence::pair;
use nom::{
    branch::alt,
    character::complete::char,
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};
use nom_locate::{self, position};
use serde::{Deserialize, Serialize};
use std::str;
type Span<'a> = nom_locate::LocatedSpan<&'a str>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SrcLoc {
    line: u32,
    offset: usize,
}

fn parse_number(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    map(double, Expr::Num)(i)
}

fn parse_string(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let pos = position::<Span, VerboseError<Span>>(i)?.1;
    let src_loc = SrcLoc {
        line: pos.location_line(),
        offset: pos.location_offset(),
    };
    map(
        delimited(char('"'), many0(is_not("\"")), char('"')),
        move |char_vec| {
            let stuff = char_vec
                .into_iter()
                .map(|x: Span| *x.fragment())
                .collect::<String>();

            Expr::String(stuff, Some(src_loc.clone()))
        },
    )(i)
}

fn parse_keyword(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    map(many1(is_not(";\n\r )(")), |char_vec| {
        match char_vec.into_iter().map(|x: Span| *x.fragment()).collect() {
            str if str == "true" => Expr::Boolean(true),
            str if str == "false" => Expr::Boolean(false),
            str => Expr::Keyword(str),
        }
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

pub fn comment(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    value(
        (), // Output is thrown away.
        pair(char(';'), many0(is_not("\n\r"))),
    )(i)
}
fn parse_list(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    map(
        delimited(char('('), many0(parse_expr), char(')')),
        make_pair_from_vec,
    )(i)
}

fn parse_quote(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    map(
        preceded(
            char('\''),
            alt((
                parse_quote,
                parse_list,
                parse_number,
                parse_string,
                parse_keyword,
            )),
        ),
        |exprs| {
            Expr::Pair(
                Box::new(Expr::Keyword("quote".to_string())),
                Box::new(Expr::Pair(Box::new(exprs), Box::new(Expr::Nil))),
            )
        },
    )(i)
}

fn parse_expr(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    delimited(
        many0(alt((comment, value((), multispace1)))),
        alt((
            parse_quote,
            parse_list,
            parse_number,
            parse_string,
            parse_keyword,
        )),
        many0(alt((comment, value((), multispace1)))),
    )(i)
}

pub fn parse(i: &str) -> Result<Vec<Expr>, String> {
    many0(parse_expr)(Span::new(i))
        .map_err(|e| format!("{e:?}"))
        .and_then(|(remaining, exp)| match *remaining.fragment() {
            "" => Ok(exp),
            remainder => Err(format!("Unexpected end of input: {remainder}")),
        })
}

#[test]
fn test_parse_alphanumerics() {
    fn kw(string: &str) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Keyword(string.to_string())])
    }
    fn nr(nr: f64) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Num(nr)])
    }

    assert_eq!(parse("true"), Ok(vec![Expr::Boolean(true)]));
    assert_eq!(parse("false"), Ok(vec![Expr::Boolean(false)]));
    assert_eq!(
        parse("\"hello world\""),
        Ok(vec![Expr::String("hello world".to_string(), None)])
    );
    assert_eq!(
        parse("(\"hello 1\" \"hello 2\" )"),
        Ok(vec![make_pair_from_vec(vec![
            Expr::String("hello 1".to_string(), None),
            Expr::String("hello 2".to_string(), None),
        ])])
    );
    assert_eq!(parse("+"), kw("+"));
    assert_eq!(parse("'"), kw("'"));
    assert_eq!(parse("1"), nr(1.));
    assert_eq!(parse("5"), nr(5.));

    assert_eq!(parse("  true  "), Ok(vec![Expr::Boolean(true)]));
    assert_eq!(parse("  false  "), Ok(vec![Expr::Boolean(false)]));
    assert_eq!(parse("  1  "), nr(1.));
    assert_eq!(parse("  5  "), nr(5.));
}

#[test]
fn test_parse_quote() {
    let res = parse("'()");
    assert_eq!(res.as_ref().map(|x| x.len()), Ok(1));

    let borrowed = res.map(|x| x.first().cloned()).unwrap().unwrap();
    matches!(borrowed, Expr::Quote(box Expr::Nil),);

    let res2 = parse("'(a b)")
        .map(|x| x.first().cloned())
        .unwrap()
        .unwrap();

    assert!(match res2 {
        Expr::Pair(
            box Expr::Keyword(quote),
            box Expr::Pair(
                box Expr::Pair(
                    box Expr::Keyword(a),
                    box Expr::Pair(box Expr::Keyword(b), box Expr::Nil),
                ),
                box Expr::Nil,
            ),
        ) => a == *"a" && b == *"b" && quote == *"quote",
        _ => false,
    });

    let res3 = parse("'a").map(|x| x.first().cloned()).unwrap().unwrap();

    assert!(match res3 {
        Expr::Pair(
            box Expr::Keyword(quote),
            box Expr::Pair(box Expr::Keyword(a), box Expr::Nil),
        ) => a == *"a" && quote == *"quote",
        _ => false,
    });
}

#[test]
fn test_parse_comment() {
    let res = parse("abc ; stuff").unwrap();
    assert_eq!(vec![Expr::Keyword("abc".to_string())], res);

    let res = parse("(abc) ; stuff").unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![Expr::Keyword("abc".to_string())])],
        res
    );

    let res = parse(
        "(abc ; comment
        cba
        ); stuff",
    )
    .unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![
            Expr::Keyword("abc".to_string()),
            Expr::Keyword("cba".to_string())
        ])],
        res
    );

    let res = parse(
        "(abc ; comment
        cba (
            hello ;abc
        )
        ); stuff",
    )
    .unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![
            Expr::Keyword("abc".to_string()),
            Expr::Keyword("cba".to_string()),
            make_pair_from_vec(vec![Expr::Keyword("hello".to_string()),]),
        ])],
        res
    );
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
