use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, multispace0, multispace1, none_of, one_of},
    combinator::{map, map_res, opt, recognize, value, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult, Parser,
};
use smol_str::SmolStr;

use super::ast::*;
use super::error::ParseErrorInfo;

type ParseResult<'a, T> = IResult<&'a str, T, ParseErrorInfo>;

fn skip_whitespace_and_comments<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    let mut pos = input;
    loop {
        let before = pos;
        while let Some(b) = pos.as_bytes().first() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => pos = &pos[1..],
                _ => break,
            }
        }
        if pos.starts_with("//") {
            let end = pos.find('\n').unwrap_or(pos.len());
            pos = &pos[end..];
            continue;
        }
        if pos.starts_with("/*") {
            let mut depth = 1usize;
            let mut i = 2;
            while depth > 0 && i < pos.len() {
                if pos[i..].starts_with("/*") { depth += 1; i += 2; }
                else if pos[i..].starts_with("*/") { depth -= 1; i += 2; }
                else { i += 1; }
            }
            pos = &pos[i..];
            continue;
        }
        if pos == before { break; }
    }
    Ok((pos, ""))
}

fn ws<'a, F, O>(inner: F) -> impl Parser<&'a str, Output = O, Error = ParseErrorInfo>
where
    F: Parser<&'a str, Output = O, Error = ParseErrorInfo>,
{
    delimited(skip_whitespace_and_comments, inner, skip_whitespace_and_comments)
}

fn keyword<'a>(s: &'static str) -> impl Parser<&'a str, Output = &'a str, Error = ParseErrorInfo> {
    terminated(tag(s), verify(multispace1, |s: &&str| !s.is_empty()))
}

fn keyword_exact<'a>(s: &'static str) -> impl Parser<&'a str, Output = &'a str, Error = ParseErrorInfo> {
    terminated(
        tag(s),
        alt((multispace1, tag("("), tag("{"), tag(")"), tag("}"), tag(":"), tag(","))),
    )
}

fn ident<'a>(input: &'a str) -> ParseResult<'a, SmolStr> {
    map(
        recognize(pair(
            alt((letter, tag("_"))),
            many0(alt((letter, digit, tag("_")))),
        )),
        |s: &str| SmolStr::new(s),
    ).parse(input)
}

fn letter<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    let mut chars = input.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => Ok((chars.as_str(), &input[..c.len_utf8()])),
        _ => Err(nom::Err::Error(ParseErrorInfo { message: "expected letter".into(), input: input.to_string(), offset: 0 })),
    }
}

fn digit<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    let mut chars = input.chars();
    match chars.next() {
        Some(c) if c.is_ascii_digit() => Ok((chars.as_str(), &input[..c.len_utf8()])),
        _ => Err(nom::Err::Error(ParseErrorInfo { message: "expected digit".into(), input: input.to_string(), offset: 0 })),
    }
}

fn bool_literal<'a>(input: &'a str) -> ParseResult<'a, AstNode> {
    alt((
        value(AstNode::BoolLiteral(true), keyword_exact("true")),
        value(AstNode::BoolLiteral(false), keyword_exact("false")),
    )).parse(input)
}

fn int_literal<'a>(input: &'a str) -> ParseResult<'a, AstNode> {
    map_res(digit1, |s: &str| {
        s.parse::<i64>().map(AstNode::IntLiteral)
    }).parse(input)
}

fn float_literal<'a>(input: &'a str) -> ParseResult<'a, AstNode> {
    map_res(
        recognize((
            digit1,
            char('.'),
            digit1,
            opt((one_of("eE"), opt(one_of("+-")), digit1)),
        )),
        |s: &str| s.parse::<f64>().map(AstNode::FloatLiteral),
    ).parse(input)
}

fn string_literal<'a>(input: &'a str) -> ParseResult<'a, AstNode> {
    map(
        delimited(
            char('"'),
            map(many0(none_of("\"")), |v: Vec<char>| v.into_iter().collect::<String>()),
            char('"'),
        ),
        |s| AstNode::StringLiteral(SmolStr::new(s)),
    ).parse(input)
}

fn type_expr<'a>(input: &'a str) -> ParseResult<'a, TypeExpr> {
    alt((
        value(TypeExpr::Infer, tag("_")),
        map(ident, TypeExpr::Named),
    )).parse(input)
}

fn param<'a>(input: &'a str) -> ParseResult<'a, Param> {
    let (input, name) = ident(input)?;
    let (input, type_expr) = match ws(tag(":")).parse(input) {
        Ok((rest, _)) => {
            let (rest, te) = type_expr(rest)?;
            (rest, Some(te))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    Ok((input, Param { name, type_expr }))
}

fn params<'a>(input: &'a str) -> ParseResult<'a, Vec<Param>> {
    delimited(
        ws(tag("(")),
        separated_list0(ws(tag(",")), param),
        ws(tag(")")),
    ).parse(input)
}

fn type_params<'a>(input: &'a str) -> ParseResult<'a, Vec<Param>> {
    params(input)
}

fn many_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, Vec<AstId>> {
    let mut items = Vec::new();
    let mut remaining = input;
    while let Ok((rest, item)) = stmt(arena, remaining) {
        items.push(item);
        remaining = rest;
    }
    Ok((remaining, items))
}

fn separated_expr_list0<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, Vec<AstId>> {
    let mut items = Vec::new();
    let mut remaining = input;
    if let Ok((rest, first)) = expr(arena, remaining) {
        items.push(first);
        remaining = rest;
        loop {
            match ws(tag(",")).parse(remaining) {
                Ok((after_comma, _)) => {
                    match expr(arena, after_comma) {
                        Ok((rest, item)) => {
                            items.push(item);
                            remaining = rest;
                        }
                        Err(_) => break,
                    }
                }
                Err(_) => break,
            }
        }
    }
    Ok((remaining, items))
}

fn parse_program_inner<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (i, _) = multispace0.parse(input)?;
    let (i, _) = keyword_exact("Program").parse(i)?;
    let (i, name) = ident(i)?;
    let (i, _) = ws(tag("{")).parse(i)?;
    let mut body_ids = Vec::new();
    let mut remaining = i;
    while let Ok((rest, item)) = stmt(arena, remaining) {
        body_ids.push(item);
        remaining = rest;
    }
    let (i, _) = ws(tag("}")).parse(remaining)?;
    let (i, _) = multispace0.parse(i)?;
    let id = arena.alloc(AstNode::Program {
        name,
        body: body_ids,
    });
    Ok((i, id))
}

pub fn parse_program(input: &str) -> Result<(AstArena, AstId), String> {
    let mut arena = AstArena::new();
    let result = parse_program_inner(&mut arena, input);
    match result {
        Ok((_, id)) => Ok((arena, id)),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let snippet = if e.offset < input.len() {
                let start = e.offset.saturating_sub(20);
                let end = (e.offset + 20).min(input.len());
                format!("...{}...", &input[start..end])
            } else {
                String::new()
            };
            Err(format!("Parse error: {} (at offset {}, near {:?})", e.message, e.offset, snippet))
        }
        Err(nom::Err::Incomplete(_)) => Err("Incomplete input".to_string()),
    }
}

fn stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = skip_whitespace_and_comments(input)?;
    var_decl(arena, input)
        .or_else(|_| if_stmt(arena, input))
        .or_else(|_| while_stmt(arena, input))
        .or_else(|_| loop_stmt(arena, input))
        .or_else(|_| return_stmt(arena, input))
        .or_else(|_| fn_def(arena, input))
        .or_else(|_| entity_def(arena, input))
        .or_else(|_| system_def(arena, input))
        .or_else(|_| actor_def(arena, input))
        .or_else(|_| view_decl(arena, input))
        .or_else(|_| state_decl(arena, input))
        .or_else(|_| block_stmt(arena, input))
        .or_else(|_| expr_stmt(arena, input))
}

fn var_decl<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = alt((keyword_exact("var"), keyword_exact("let"), keyword_exact("const"))).parse(input)?;
    let mutable = true;
    let (input, name) = ident(input)?;
    let (input, type_expr) = match ws(tag(":")).parse(input) {
        Ok((rest, _)) => {
            let (rest, ty) = type_expr(rest)?;
            (rest, Some(ty))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    let (input, init) = match ws(tag("=")).parse(input) {
        Ok((rest, _)) => {
            let (rest, val) = expr(arena, rest)?;
            (rest, Some(val))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    let (input, _) = ws(tag(";")).parse(input)?;

    Ok((input, arena.alloc(AstNode::VarDecl {
        name,
        type_expr,
        init,
        mutable,
    })))
}

fn if_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("if").parse(input)?;
    let (input, _) = ws(tag("(")).parse(input)?;
    let (input, cond) = expr(arena, input)?;
    let (input, _) = ws(tag(")")).parse(input)?;
    let (input, then_block) = block_inner(arena, input)?;
    let (input, else_block) = match keyword_exact("else").parse(input) {
        Ok((rest, _)) => {
            let (rest, block) = block_inner(arena, rest)?;
            (rest, Some(block))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };

    Ok((input, arena.alloc(AstNode::If {
        cond,
        then_block,
        else_block,
    })))
}

fn while_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("while").parse(input)?;
    let (input, _) = ws(tag("(")).parse(input)?;
    let (input, cond) = expr(arena, input)?;
    let (input, _) = ws(tag(")")).parse(input)?;
    let (input, body) = block_inner(arena, input)?;

    Ok((input, arena.alloc(AstNode::While { cond, body })))
}

fn loop_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("Loop").parse(input)?;
    let (input, var) = ident(input)?;
    let (input, _) = ws(keyword_exact("from")).parse(input)?;
    let (input, from) = expr(arena, input)?;
    let (input, _) = ws(keyword_exact("to")).parse(input)?;
    let (input, to) = expr(arena, input)?;
    let (input, _) = ws(tag(")")).parse(input)?;
    let (input, body) = block_inner(arena, input)?;

    Ok((input, arena.alloc(AstNode::Loop { var, from, to, body })))
}

fn return_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("Return").parse(input)?;
    let (input, value) = match expr(arena, input) {
        Ok((rest, val)) => (rest, Some(val)),
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    let (input, _) = ws(tag(";")).parse(input)?;

    Ok((input, arena.alloc(AstNode::Return(value))))
}

fn expr_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, e) = expr(arena, input)?;
    let (input, _) = opt(ws(tag(";"))).parse(input)?;
    Ok((input, e))
}

fn block_inner<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = ws(tag("{")).parse(input)?;
    let (input, stmts) = many_stmt(arena, input)?;
    let (input, _) = ws(tag("}")).parse(input)?;

    Ok((input, arena.alloc(AstNode::Block(stmts))))
}

fn block_stmt<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    block_inner(arena, input)
}

fn fn_def<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, is_async) = match keyword_exact("async").parse(input) {
        Ok((rest, _)) => (rest, true),
        Err(nom::Err::Error(_)) => (input, false),
        Err(e) => return Err(e),
    };
    let (input, is_diff) = match keyword_exact("differentiable").parse(input) {
        Ok((rest, _)) => (rest, true),
        Err(nom::Err::Error(_)) => (input, false),
        Err(e) => return Err(e),
    };
    let (input, _) = keyword_exact("Function").parse(input)?;
    let (input, name) = ident(input)?;
    let (input, params) = params(input)?;
    let (input, ret_type) = match ws(tag("->")).parse(input) {
        Ok((rest, _)) => {
            let (rest, ty) = type_expr(rest)?;
            (rest, Some(ty))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    let (input, body) = block_inner(arena, input)?;

    Ok((input, arena.alloc(AstNode::Function {
        name,
        params,
        ret_type,
        body: vec![body],
        is_async,
        is_differentiable: is_diff,
    })))
}

fn entity_def<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("Entity").parse(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = ws(tag("{")).parse(input)?;

    let mut components = Vec::new();
    let mut remaining = input;
    loop {
        match stmt(arena, remaining) {
            Ok((rest, comp)) => {
                components.push(comp);
                remaining = rest;
            }
            Err(_) => break,
        }
    }

    let (input, _) = ws(tag("}")).parse(remaining)?;

    Ok((input, arena.alloc(AstNode::EntityDef { name, components })))
}

fn system_def<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("System").parse(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = ws(tag("(")).parse(input)?;
    let (input, query) = ident(input)?;
    let (input, _) = ws(tag(")")).parse(input)?;
    let (input, body) = block_inner(arena, input)?;

    Ok((input, arena.alloc(AstNode::SystemDef { name, query, body })))
}

fn actor_def<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("Actor").parse(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = ws(tag("{")).parse(input)?;

    let mut handlers = Vec::new();
    let mut remaining = input;
    loop {
        match keyword_exact("On").parse(remaining) {
            Ok((rest, _)) => {
                let (rest, _) = ws(tag("(")).parse(rest)?;
                let (rest, event) = ident(rest)?;
                let (rest, _) = ws(tag(")")).parse(rest)?;
                let (rest, body) = block_inner(arena, rest)?;
                let (rest, _) = ws(tag(";")).parse(rest)?;
                handlers.push(arena.alloc(AstNode::OnHandler { event, body }));
                remaining = rest;
            }
            Err(_) => break,
        }
    }

    let (input, _) = ws(tag("}")).parse(remaining)?;

    Ok((input, arena.alloc(AstNode::ActorDef { name, handlers })))
}

fn view_decl<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("View").parse(input)?;
    let (input, _) = ws(tag("{")).parse(input)?;
    let (input, children) = many_stmt(arena, input)?;
    let (input, _) = ws(tag("}")).parse(input)?;

    Ok((input, arena.alloc(AstNode::View { children })))
}

fn state_decl<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, _) = keyword_exact("State").parse(input)?;
    let (input, _) = ws(tag("<")).parse(input)?;
    let (input, type_expr) = type_expr(input)?;
    let (input, _) = ws(tag(">")).parse(input)?;
    let (input, name) = ident(input)?;
    let (input, init) = match ws(tag("=")).parse(input) {
        Ok((rest, _)) => {
            let (rest, val) = expr(arena, rest)?;
            (rest, Some(val))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    let (input, _) = ws(tag(";")).parse(input)?;

    Ok((input, arena.alloc(AstNode::StateDecl {
        name,
        type_expr: Some(type_expr),
        init,
    })))
}

fn expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    or_expr(arena, input)
}

fn or_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, left) = and_expr(arena, input)?;
    let (input, right) = match ws(tag("||")).parse(input) {
        Ok((rest, _)) => {
            let (rest, r) = or_expr(arena, rest)?;
            (rest, Some(r))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    match right {
        Some(r) => Ok((input, arena.alloc(AstNode::Binary { op: BinOp::Or, left, right: r }))),
        None => Ok((input, left)),
    }
}

fn and_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, left) = cmp_expr(arena, input)?;
    let (input, right) = match ws(tag("&&")).parse(input) {
        Ok((rest, _)) => {
            let (rest, r) = and_expr(arena, rest)?;
            (rest, Some(r))
        }
        Err(nom::Err::Error(_)) => (input, None),
        Err(e) => return Err(e),
    };
    match right {
        Some(r) => Ok((input, arena.alloc(AstNode::Binary { op: BinOp::And, left, right: r }))),
        None => Ok((input, left)),
    }
}

fn cmp_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, left) = add_expr(arena, input)?;
    let (input, op) = try_cmp_op(input)?;
    match op {
        Some(op) => {
            let (input, right) = add_expr(arena, input)?;
            Ok((input, arena.alloc(AstNode::Binary { op, left, right })))
        }
        None => Ok((input, left)),
    }
}

fn try_cmp_op<'a>(input: &'a str) -> ParseResult<'a, Option<BinOp>> {
    alt((
        value(BinOp::Eq, ws(tag("=="))),
        value(BinOp::Neq, ws(tag("!="))),
        value(BinOp::Le, ws(tag("<="))),
        value(BinOp::Ge, ws(tag(">="))),
        value(BinOp::Lt, ws(tag("<"))),
        value(BinOp::Gt, ws(tag(">"))),
    )).parse(input).map(|(i, op)| (i, Some(op)))
     .or_else(|_: nom::Err<ParseErrorInfo>| Ok((input, None)))
}

fn add_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, left) = mul_expr(arena, input)?;
    let (input, op_and_right) = try_add_op(arena, input)?;
    match op_and_right {
        Some((op, right)) => Ok((input, arena.alloc(AstNode::Binary { op, left, right }))),
        None => Ok((input, left)),
    }
}

fn try_add_op<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, Option<(BinOp, AstId)>> {
    if let Ok((rest, _)) = value(BinOp::Add, ws(tag("+"))).parse(input) {
        if let Ok((rest, right)) = mul_expr(arena, rest) {
            return Ok((rest, Some((BinOp::Add, right))));
        }
    }
    if let Ok((rest, _)) = value(BinOp::Sub, ws(tag("-"))).parse(input) {
        if let Ok((rest, right)) = mul_expr(arena, rest) {
            return Ok((rest, Some((BinOp::Sub, right))));
        }
    }
    Ok((input, None))
}

fn mul_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, left) = unary_expr(arena, input)?;
    let (input, op_and_right) = try_mul_op(arena, input)?;
    match op_and_right {
        Some((op, right)) => Ok((input, arena.alloc(AstNode::Binary { op, left, right }))),
        None => Ok((input, left)),
    }
}

fn try_mul_op<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, Option<(BinOp, AstId)>> {
    if let Ok((rest, _)) = value(BinOp::Mul, ws(tag("*"))).parse(input) {
        if let Ok((rest, right)) = unary_expr(arena, rest) {
            return Ok((rest, Some((BinOp::Mul, right))));
        }
    }
    if let Ok((rest, _)) = value(BinOp::Div, ws(tag("/"))).parse(input) {
        if let Ok((rest, right)) = unary_expr(arena, rest) {
            return Ok((rest, Some((BinOp::Div, right))));
        }
    }
    if let Ok((rest, _)) = value(BinOp::Mod, ws(tag("%"))).parse(input) {
        if let Ok((rest, right)) = unary_expr(arena, rest) {
            return Ok((rest, Some((BinOp::Mod, right))));
        }
    }
    Ok((input, None))
}

fn unary_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    if let Ok((rest, _)) = value(UnaryOp::Neg, ws(tag("-"))).parse(input) {
        if let Ok((rest, operand)) = unary_expr(arena, rest) {
            return Ok((rest, arena.alloc(AstNode::Unary { op: UnaryOp::Neg, operand })));
        }
    }
    if let Ok((rest, _)) = value(UnaryOp::Not, ws(tag("!"))).parse(input) {
        if let Ok((rest, operand)) = unary_expr(arena, rest) {
            return Ok((rest, arena.alloc(AstNode::Unary { op: UnaryOp::Not, operand })));
        }
    }
    call_expr(arena, input)
}

fn call_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    let (input, callee) = primary_expr(arena, input)?;

    let mut current = callee;
    let mut remaining = input;

    loop {
        if let Ok((rest, _)) = tag::<_, _, ParseErrorInfo>(".").parse(remaining) {
            if let Ok((rest2, method)) = ident(rest) {
                let method_call = (|| -> Result<_, nom::Err<ParseErrorInfo>> {
                    let (i, _) = ws(tag("(")).parse(rest2)?;
                    let (i, args) = separated_expr_list0(arena, i)?;
                    let (i, _) = ws(tag(")")).parse(i)?;
                    Ok((i, args))
                })();
                if let Ok((rest3, args)) = method_call {
                    current = arena.alloc(AstNode::MethodCall {
                        object: current,
                        method,
                        args,
                    });
                    remaining = rest3;
                    continue;
                }
                current = arena.alloc(AstNode::FieldAccess {
                    object: current,
                    field: method,
                });
                remaining = rest2;
                continue;
            }
        }

        let call_result = (|| -> Result<_, nom::Err<ParseErrorInfo>> {
            let (i, _) = ws(tag("(")).parse(remaining)?;
            let (i, args) = separated_expr_list0(arena, i)?;
            let (i, _) = ws(tag(")")).parse(i)?;
            Ok((i, args))
        })();
        if let Ok((rest, args)) = call_result {
            current = arena.alloc(AstNode::Call {
                callee: current,
                args,
            });
            remaining = rest;
            continue;
        }

        let index_result = (|| -> Result<_, nom::Err<ParseErrorInfo>> {
            let (i, _) = ws(tag("[")).parse(remaining)?;
            let (i, index) = expr(arena, i)?;
            let (i, _) = ws(tag("]")).parse(i)?;
            Ok((i, index))
        })();
        if let Ok((rest, index)) = index_result {
            current = arena.alloc(AstNode::Index {
                object: current,
                index,
            });
            remaining = rest;
            continue;
        }

        break;
    }

    Ok((remaining, current))
}

fn primary_expr<'a>(arena: &mut AstArena, input: &'a str) -> ParseResult<'a, AstId> {
    if let Ok((rest, n)) = float_literal(input) {
        return Ok((rest, arena.alloc(n)));
    }
    if let Ok((rest, n)) = int_literal(input) {
        return Ok((rest, arena.alloc(n)));
    }
    if let Ok((rest, n)) = bool_literal(input) {
        return Ok((rest, arena.alloc(n)));
    }
    if let Ok((rest, n)) = string_literal(input) {
        return Ok((rest, arena.alloc(n)));
    }
    if let Ok((rest, name)) = ident(input) {
        return Ok((rest, arena.alloc(AstNode::Identifier(name))));
    }
    if let Ok((rest, _)) = ws(tag("(")).parse(input) {
        let (rest, e) = expr(arena, rest)?;
        let (rest, _) = ws(tag(")")).parse(rest)?;
        return Ok((rest, e));
    }
    Err(nom::Err::Error(ParseErrorInfo {
        message: "expected expression".to_string(),
        input: input.to_string(),
        offset: input.len(),
    }))
}
