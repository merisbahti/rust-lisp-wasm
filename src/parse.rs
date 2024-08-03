use crate::expr::Expr;
use nom::bytes::complete::{is_not, tag};
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
use std::fmt::Display;
use std::str;
type Span<'a> = nom_locate::LocatedSpan<&'a str, Option<&'a str>>;

#[derive(Clone, Debug)]
pub struct SrcLoc {
    line: u32,
    offset: usize,
    file_name: Option<String>,
}

impl Display for SrcLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file_name.clone().unwrap_or("unknown".to_string()),
            self.line,
            self.offset
        )
    }
}

fn parse_number(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    map(double, Expr::Num)(i)
}

fn parse_string(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let file_name = i.extra.map(|x| x.to_string());
    let pos = position::<Span, VerboseError<Span>>(i)?.1;
    let src_loc = SrcLoc {
        line: pos.location_line(),
        offset: pos.location_offset(),
        file_name,
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
    let pos = position::<Span, VerboseError<Span>>(i)?.1;
    let file_name = i.extra.map(|x| x.to_string());
    let src_loc = SrcLoc {
        line: pos.location_line(),
        offset: pos.location_offset(),
        file_name,
    };

    map(many1(is_not(";\n\r )(")), move |char_vec| {
        match char_vec.into_iter().map(|x: Span| *x.fragment()).collect() {
            str if str == "true" => Expr::Boolean(true),
            str if str == "false" => Expr::Boolean(false),
            str => Expr::Keyword(str, Some(src_loc.clone())),
        }
    })(i)
}

pub fn make_pair_from_vec(v: Vec<Expr>) -> Expr {
    match v.split_first() {
        Some((head, tail)) => Expr::Pair(
            Box::new(head.clone()),
            Box::new(make_pair_from_vec(tail.to_vec()).clone()),
            Some(SrcLoc {
                line: 13371337,
                offset: 13371337,
                file_name: None,
            }),
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
fn parse_pair(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (mut i, _) = tag("(")(i)?;
    let mut results: Vec<(SrcLoc, Expr)> = Vec::new();
    while let Ok((new_s, expr)) = parse_expr(i) {
        i = new_s;
        let pos = position::<Span, VerboseError<Span>>(i)?.0;
        let file_name = i.extra.map(|x| x.to_string());
        let src_loc = SrcLoc {
            line: pos.location_line(),
            offset: pos.location_offset(),
            file_name,
        };
        results.push((src_loc, expr))
    }
    let (i, _) = tag(")")(i)?;

    let res =
        results
            .into_iter()
            .rev()
            .fold(Expr::Nil, |acc: Expr, (loc, expr): (SrcLoc, Expr)| {
                Expr::Pair(Box::new(expr.clone()), Box::new(acc), Some(loc.clone()))
            });
    Ok((i, res))
}

fn parse_quote(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let pos = position::<Span, VerboseError<Span>>(i)?.1;
    let file_name = i.extra.map(|x| x.to_string());
    let src_loc = SrcLoc {
        line: pos.location_line(),
        offset: pos.location_offset(),
        file_name: file_name.clone(),
    };

    map(
        preceded(
            char('\''),
            alt((
                parse_quote,
                parse_pair,
                parse_number,
                parse_string,
                parse_keyword,
            )),
        ),
        move |exprs| {
            Expr::Pair(
                Box::new(Expr::Keyword(
                    "quote".to_string(),
                    Some(SrcLoc {
                        line: src_loc.line,
                        offset: src_loc.offset,
                        file_name: file_name.clone(),
                    }),
                )),
                Box::new(Expr::Pair(
                    Box::new(exprs),
                    Box::new(Expr::Nil),
                    Some(SrcLoc {
                        line: src_loc.line,
                        offset: src_loc.offset,
                        file_name: file_name.clone(),
                    }),
                )),
                Some((&src_loc).clone()),
            )
        },
    )(i)
}

fn parse_expr(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    delimited(
        many0(alt((comment, value((), multispace1)))),
        alt((
            parse_quote,
            parse_pair,
            parse_number,
            parse_string,
            parse_keyword,
        )),
        many0(alt((comment, value((), multispace1)))),
    )(i)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ParseInput<'a> {
    pub source: &'a str,
    pub file_name: Option<&'a str>,
}

pub fn parse(input: &ParseInput) -> Result<Vec<Expr>, String> {
    many0(parse_expr)(Span::new_extra(input.source, input.file_name))
        .map_err(|e| format!("{e:?}"))
        .and_then(|(remaining, exp)| match *remaining.fragment() {
            "" => Ok(exp),
            remainder => Err(format!("Unexpected end of input: {remainder}")),
        })
}

#[test]
fn test_parse_alphanumerics() {
    fn kw(string: &str) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Keyword(string.to_string(), None)])
    }
    fn nr(nr: f64) -> Result<Vec<Expr>, String> {
        Ok(vec![Expr::Num(nr)])
    }

    assert_eq!(
        parse(&ParseInput {
            source: "true",
            file_name: None
        }),
        Ok(vec![Expr::Boolean(true)])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "false",
            file_name: None
        }),
        Ok(vec![Expr::Boolean(false)])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "\"hello world\"",
            file_name: None
        }),
        Ok(vec![Expr::String("hello world".to_string(), None)])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "(\"hello 1\" \"hello 2\" )",
            file_name: None
        }),
        Ok(vec![make_pair_from_vec(vec![
            Expr::String("hello 1".to_string(), None),
            Expr::String("hello 2".to_string(), None),
        ])])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "+",
            file_name: None
        }),
        kw("+")
    );
    assert_eq!(
        parse(&ParseInput {
            source: "'",
            file_name: None
        }),
        kw("'")
    );
    assert_eq!(
        parse(&ParseInput {
            source: "1",
            file_name: None
        }),
        nr(1.)
    );
    assert_eq!(
        parse(&ParseInput {
            source: "5",
            file_name: None
        }),
        nr(5.)
    );

    assert_eq!(
        parse(&ParseInput {
            source: "  true  ",
            file_name: None
        }),
        Ok(vec![Expr::Boolean(true)])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "  false  ",
            file_name: None
        }),
        Ok(vec![Expr::Boolean(false)])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "  1  ",
            file_name: None
        }),
        nr(1.)
    );
    assert_eq!(
        parse(&ParseInput {
            source: "  5  ",
            file_name: None
        }),
        nr(5.)
    );
}

#[test]
fn test_parse_quote() {
    let res = parse(&ParseInput {
        source: "'()",
        file_name: None,
    });
    assert_eq!(res.as_ref().map(|x| x.len()), Ok(1));

    let borrowed = res.map(|x| x.first().cloned()).unwrap().unwrap();
    matches!(
        borrowed,
        Expr::Quote(
            box Expr::Nil,
            Some(SrcLoc {
                line: 0,
                offset: 0,
                file_name: None,
            }),
        ),
    );

    let res2 = parse(&ParseInput {
        source: "'(a b)",
        file_name: None,
    })
    .map(|x| x.first().cloned())
    .unwrap()
    .unwrap();

    assert_matches!(res2,
        Expr::Pair(
            box Expr::Keyword(quote, ..),
            box Expr::Pair(
                box Expr::Pair(
                    box Expr::Keyword(a,..),
                    box Expr::Pair(box Expr::Keyword(b,..), box Expr::Nil, ..),
                    ..,
                ),
                box Expr::Nil,
                ..,
            ),
            ..,
        ) if  a == *"a" && b == *"b" && quote == *"quote",
    );

    let res3 = parse(&ParseInput {
        source: "'a",
        file_name: None,
    })
    .map(|x| x.first().cloned())
    .unwrap()
    .unwrap();

    use std::assert_matches::assert_matches;
    assert_matches!(res3,
        Expr::Pair(
            box Expr::Keyword(quote, ..),
            box Expr::Pair(box Expr::Keyword(a, ..), box Expr::Nil, ..),
            ..,
        ) if a == *"a" && quote == *"quote",
    );
}

#[test]
fn test_parse_comment() {
    let res = parse(&ParseInput {
        source: "abc ; stuff",
        file_name: None,
    })
    .unwrap();
    assert_eq!(vec![Expr::Keyword("abc".to_string(), None)], res);

    let res = parse(&ParseInput {
        source: "(abc) ; stuff",
        file_name: None,
    })
    .unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![Expr::Keyword(
            "abc".to_string(),
            None
        )])],
        res
    );

    let res = parse(&ParseInput {
        source: "(abc ; comment
        cba
        ); stuff",
        file_name: None,
    })
    .unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![
            Expr::Keyword("abc".to_string(), None),
            Expr::Keyword("cba".to_string(), None)
        ])],
        res
    );

    let res = parse(&ParseInput {
        source: "(abc ; comment
        cba (
            hello ;abc
        )
        ); stuff",
        file_name: None,
    })
    .unwrap();
    assert_eq!(
        vec![make_pair_from_vec(vec![
            Expr::Keyword("abc".to_string(), None),
            Expr::Keyword("cba".to_string(), None),
            make_pair_from_vec(vec![Expr::Keyword("hello".to_string(), None),]),
        ])],
        res
    );
}
#[test]
fn test_parse_lists() {
    use std::assert_matches::assert_matches;
    fn ok_list(strings: Vec<&str>) -> Result<Vec<Expr>, String> {
        let stuff: Vec<Expr> = strings
            .iter()
            .map(|x| Expr::Keyword(x.to_string(), None))
            .collect();
        Ok(vec![make_pair_from_vec(stuff)])
    }
    assert_eq!(
        parse(&ParseInput {
            source: "(a)",
            file_name: None
        }),
        ok_list(vec!(("a")))
    );
    assert_matches!(
        parse(&ParseInput {
            source: "(123)",
            file_name: None
        })
        .map(|x| x.split_first().map(|x| x.0.clone())),
        Ok(Some(Expr::Pair(box Expr::Num(123.), box Expr::Nil, ..)))
    );
    assert_eq!(
        parse(&ParseInput {
            source: "(     a         1       b          2      )",
            file_name: None
        }),
        Ok(vec![make_pair_from_vec(vec![
            Expr::Keyword("a".to_string(), None),
            Expr::Num(1.),
            Expr::Keyword("b".to_string(), None),
            Expr::Num(2.)
        ])])
    );
    assert_eq!(
        parse(&ParseInput {
            source: "(     a         (wat    woo    wii)       b          2      )",
            file_name: None
        }),
        Ok(vec![make_pair_from_vec(vec![
            Expr::Keyword("a".to_string(), None),
            make_pair_from_vec(vec!(
                Expr::Keyword("wat".to_string(), None),
                Expr::Keyword("woo".to_string(), None),
                Expr::Keyword("wii".to_string(), None)
            )),
            Expr::Keyword("b".to_string(), None),
            Expr::Num(2.)
        ])])
    );
}
