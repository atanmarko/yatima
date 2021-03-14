// use directories_next::ProjectDirs;
use rustyline::{
  error::ReadlineError,
  Cmd,
  Config,
  EditMode,
  Editor,
  KeyEvent,
};

use im::{
  HashMap,
  Vector,
};

use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::multispace0,
  combinator::value,
  sequence::terminated,
  Err,
  IResult,
};

use std::path::PathBuf;

use crate::{
  core::{
    dag::DAG,
    eval::norm,
  },
  package::Declaration,
  parse::{
    error::ParseError,
    package::{
      parse_defn,
      parse_open,
      PackageEnv,
    },
    span::Span,
    term::{
      parse,
      parse_expression,
    },
  },
  term::{
    Defs,
    Refs,
    Term,
  },
};

pub enum Command {
  Eval(Term),
  Type(Term),
  Load(PathBuf),
  Decl(Declaration),
  Browse,
  Help,
  Quit,
}

pub fn parse_decl(
  defs: Defs,
  refs: Refs,
  env: PackageEnv,
) -> impl Fn(Span) -> IResult<Span, (Command, Refs, Defs), ParseError<Span>> {
  move |i: Span| {
    let (i2, (decl, new_refs, new_defs)) = alt((
      parse_defn(refs.clone(), defs.clone()),
      parse_open(refs.clone(), defs.clone(), env.to_owned()),
    ))(i)?;
    Ok((i2, (Command::Decl(decl), new_refs, new_defs)))
  }
}

pub fn parse_eval(
  defs: Defs,
  refs: Refs,
) -> impl Fn(Span) -> IResult<Span, (Command, Refs, Defs), ParseError<Span>> {
  move |i: Span| {
    let (i2, term) = parse_expression(refs.clone(), Vector::new())(i)?;
    Ok((i2, (Command::Eval(term), refs.to_owned(), defs.to_owned())))
  }
}
pub fn parse_browse(
  defs: Defs,
  refs: Refs,
) -> impl Fn(Span) -> IResult<Span, (Command, Refs, Defs), ParseError<Span>> {
  move |i: Span| {
    let (i2, _) = terminated(tag(":browse"), multispace0)(i)?;
    Ok((i2, (Command::Browse, refs.to_owned(), defs.to_owned())))
  }
}

pub fn parse_quit(
  defs: Defs,
  refs: Refs,
) -> impl Fn(Span) -> IResult<Span, (Command, Refs, Defs), ParseError<Span>> {
  move |i: Span| {
    let (i2, _) = terminated(tag(":quit"), multispace0)(i)?;
    Ok((i2, (Command::Quit, refs.to_owned(), defs.to_owned())))
  }
}

pub fn main() -> rustyline::Result<()> {
  let config = Config::builder().edit_mode(EditMode::Vi).build();
  let mut rl = Editor::<()>::with_config(config);
  let mut decls: Vec<Declaration> = Vec::new();
  let mut refs: Refs = HashMap::new();
  let mut defs: Defs = HashMap::new();
  let env: PackageEnv = PackageEnv::new(PathBuf::from("~/repl_tmp"));
  rl.bind_sequence(KeyEvent::alt('l'), Cmd::Insert(1, String::from("λ ")));
  rl.bind_sequence(KeyEvent::alt('a'), Cmd::Insert(1, String::from("∀ ")));
  if rl.load_history("history.txt").is_err() {
    println!("No previous history.");
  }
  loop {
    let readline = rl.readline("⅄ ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str());
        let res = alt((
          parse_browse(defs.clone(), refs.clone()),
          parse_quit(defs.clone(), refs.clone()),
          parse_decl(defs.clone(), refs.clone(), env.clone()),
          parse_eval(defs.clone(), refs.clone()),
        ))(Span::new(&line));
        match res {
          Ok((_, (command, new_refs, new_defs))) => {
            refs = new_refs;
            defs = new_defs;
            match command {
              Command::Eval(term) => {
                println!("{}", norm(&defs, DAG::from_term(term)))
              }
              Command::Quit => {
                println!("Goodbye.");
                break;
              }
              Command::Browse => {
                for n in refs.keys() {
                  println!("{}", n)
                }
              }
              Command::Decl(decl) => decls.push(decl),
              _ => panic!("todo repl command"),
            }
          }
          Err(e) => match e {
            Err::Incomplete(_) => println!("Incomplete"),
            Err::Failure(e) => {
              println!("Parse Failure:\n");
              println!("{}", e);
            }
            Err::Error(e) => {
              println!("Parse Error:\n");
              println!("{}", e);
            }
          },
        }
      }
      Err(ReadlineError::Interrupted) => {
        println!("CTRL-C");
        break;
      }
      Err(ReadlineError::Eof) => {
        println!("CTRL-D");
        println!("Goodbye.");
        break;
      }
      Err(err) => {
        println!("Error: {}", err);
        break;
      }
    }
  }
  rl.save_history("history.txt")
}
