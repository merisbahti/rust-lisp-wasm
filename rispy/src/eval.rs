use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, multispace0, multispace1, one_of},
    combinator::{cut, map, map_res, opt},
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult, Parser,
};

/// Starting from the most basic, we define some built-in functions that our lisp has
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BuiltIn {
    Plus,
    Minus,
    Times,
    Divide,
    Equal,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Num(i32),
    Keyword(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Constant(Atom),
    /// (func-name arg1 arg2)
    Application(Box<Expr>, Vec<Expr>),
    /// (if predicate do-this)
    If(Box<Expr>, Box<Expr>),
    /// (if predicate do-this otherwise-do-this)
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
    /// '(3 (if (+ 3 3) 4 5) 7)
    Quote(Vec<Expr>),
}

fn parse_builtin_op<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    let (i, t) = one_of("+-*/=")(i)?;

    Ok((
        i,
        match t {
            '+' => BuiltIn::Plus,
            '-' => BuiltIn::Minus,
            '*' => BuiltIn::Times,
            '/' => BuiltIn::Divide,
            '=' => BuiltIn::Equal,
            _ => unreachable!(),
        },
    ))
}

fn parse_builtin<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((parse_builtin_op, map(tag("not"), |_| BuiltIn::Not)))(i)
}

fn parse_bool<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
        map(tag("#t"), |_| Atom::Boolean(true)),
        map(tag("#f"), |_| Atom::Boolean(false)),
    ))(i)
}

fn parse_keyword<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
        context("keyword", preceded(tag(":"), cut(alpha1))),
        |sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )(i)
}

fn parse_num<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
        map_res(digit1, |digit_str: &str| {
            digit_str.parse::<i32>().map(Atom::Num)
        }),
        map(preceded(tag("-"), digit1), |digit_str: &str| {
            Atom::Num(-1 * digit_str.parse::<i32>().unwrap())
        }),
    ))(i)
}

fn parse_atom<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
        parse_num,
        parse_bool,
        map(parse_builtin, Atom::BuiltIn),
        parse_keyword,
    ))(i)
}

fn parse_constant<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_atom, |atom| Expr::Constant(atom))(i)
}

fn s_exp<'a, O1, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Parser<&'a str, O1, VerboseError<&'a str>>,
{
    delimited(
        char('('),
        preceded(multispace0, inner),
        context("closing paren", cut(preceded(multispace0, char(')')))),
    )
}

fn parse_application<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    let application_inner = map(tuple((parse_expr, many0(parse_expr))), |(head, tail)| {
        Expr::Application(Box::new(head), tail)
    });
    s_exp(application_inner)(i)
}

fn parse_if<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    let if_inner = context(
        "if expression",
        map(
            preceded(
                terminated(tag("if"), multispace1),
                cut(tuple((parse_expr, parse_expr, opt(parse_expr)))),
            ),
            |(pred, true_branch, maybe_false_branch)| {
                if let Some(false_branch) = maybe_false_branch {
                    Expr::IfElse(
                        Box::new(pred),
                        Box::new(true_branch),
                        Box::new(false_branch),
                    )
                } else {
                    Expr::If(Box::new(pred), Box::new(true_branch))
                }
            },
        ),
    );
    s_exp(if_inner)(i)
}

fn parse_quote<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(
        context("quote", preceded(tag("'"), cut(s_exp(many0(parse_expr))))),
        |exprs| Expr::Quote(exprs),
    )(i)
}

fn parse_expr<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    preceded(
        multispace0,
        alt((parse_constant, parse_application, parse_if, parse_quote)),
    )(i)
}

fn get_num_from_expr(e: Expr) -> Option<i32> {
    if let Expr::Constant(Atom::Num(n)) = e {
        Some(n)
    } else {
        None
    }
}

fn get_bool_from_expr(e: Expr) -> Option<bool> {
    if let Expr::Constant(Atom::Boolean(b)) = e {
        Some(b)
    } else {
        None
    }
}

fn eval_expression(e: Expr) -> Option<Expr> {
    match e {
        Expr::Constant(_) | Expr::Quote(_) => Some(e),
        Expr::If(pred, true_branch) => {
            let reduce_pred = eval_expression(*pred)?;
            if get_bool_from_expr(reduce_pred)? {
                eval_expression(*true_branch)
            } else {
                None
            }
        }
        Expr::IfElse(pred, true_branch, false_branch) => {
            let reduce_pred = eval_expression(*pred)?;
            if get_bool_from_expr(reduce_pred)? {
                eval_expression(*true_branch)
            } else {
                eval_expression(*false_branch)
            }
        }
        Expr::Application(head, tail) => {
            let reduced_head = eval_expression(*head)?;
            let reduced_tail = tail
                .into_iter()
                .map(|expr| eval_expression(expr))
                .collect::<Option<Vec<Expr>>>()?;
            if let Expr::Constant(Atom::BuiltIn(bi)) = reduced_head {
                Some(Expr::Constant(match bi {
                    BuiltIn::Plus => Atom::Num(
                        reduced_tail
                            .into_iter()
                            .map(get_num_from_expr)
                            .collect::<Option<Vec<i32>>>()?
                            .into_iter()
                            .sum(),
                    ),
                    BuiltIn::Times => Atom::Num(
                        reduced_tail
                            .into_iter()
                            .map(get_num_from_expr)
                            .collect::<Option<Vec<i32>>>()?
                            .into_iter()
                            .product(),
                    ),
                    BuiltIn::Equal => Atom::Boolean(
                        reduced_tail
                            .iter()
                            .zip(reduced_tail.iter().skip(1))
                            .all(|(a, b)| a == b),
                    ),
                    BuiltIn::Not => {
                        if reduced_tail.len() != 1 {
                            return None;
                        } else {
                            Atom::Boolean(!get_bool_from_expr(
                                reduced_tail.first().cloned().unwrap(),
                            )?)
                        }
                    }
                    BuiltIn::Minus => {
                        Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_num_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_num_from_expr)
                                .collect::<Option<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a - b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Divide => {
                        Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_num_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_num_from_expr)
                                .collect::<Option<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a / b)
                        } else {
                            Default::default()
                        })
                    }
                }))
            } else {
                None
            }
        }
    }
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse_expr(src)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
        .and_then(|(_, exp)| eval_expression(exp).ok_or("Eval failed".to_string()))
}
