use std::{collections::HashSet, env, fs, path::PathBuf};

use full_moon::{
    self, ast,
    tokenizer::{self, Symbol, Token, TokenReference, TokenType},
    visitors::VisitorMut,
    ShortString,
};
use rayon::prelude::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
struct FunctionCallVisitor {}

impl VisitorMut for FunctionCallVisitor {
    fn visit_function_call(&mut self, node: ast::FunctionCall) -> ast::FunctionCall {
        if let ast::Prefix::Name(name_token_reference) = node.prefix() {
            if let TokenType::Identifier { identifier } = name_token_reference.token_type() {
                if identifier.as_str() != "assert" {
                    return node;
                }

                let mut suffixes = node.suffixes().cloned();
                let mut new_suffixes: Vec<ast::Suffix> = Vec::new();

                let mut first_suffix = suffixes.nth(0).unwrap().clone();

                if let ast::Suffix::Call(ast::Call::AnonymousCall(
                    ast::FunctionArgs::Parentheses {
                        ref mut arguments, ..
                    },
                )) = first_suffix
                {
                    let first_argument = arguments.iter_mut().nth(0).unwrap().clone();
                    let second_argument = arguments.iter_mut().nth(1);
                    match second_argument {
                        Some(expression) => match expression {
                            ast::Expression::String(string_token_reference) => {
                                let string = format!("{}", string_token_reference);
                                if string == "''"
                                    || string == "\"\""
                                    || string[1..].starts_with("[blam]")
                                {
                                    let token_type = TokenType::StringLiteral {
                                        literal: ShortString::new(
                                            format!("[blam]\n{}", first_argument)
                                                .replace("\\", "\\\\")
                                                .replace("\"", "\\\"")
                                                .replace("\n", "\\n")
                                                .replace("\r", "\\r")
                                                .replace("\t", "\\t"),
                                        ),
                                        multi_line: None,
                                        quote_type: tokenizer::StringLiteralQuoteType::Double,
                                    };
                                    let token = Token::new(token_type);
                                    let empty_trivia: Vec<Token> = Vec::new();
                                    let token_reference = TokenReference::new(
                                        empty_trivia.clone(),
                                        token,
                                        empty_trivia.clone(),
                                    );
                                    *expression = ast::Expression::String(token_reference);
                                }
                            }
                            ast::Expression::InterpolatedString(interpolated_string) => {
                                if format!("{}", interpolated_string) == "``" {
                                    let token_type = TokenType::StringLiteral {
                                        literal: ShortString::new(
                                            format!("[blam]\n{}", first_argument)
                                                .replace("\\", "\\\\")
                                                .replace("\"", "\\\"")
                                                .replace("\n", "\\n")
                                                .replace("\r", "\\r")
                                                .replace("\t", "\\t"),
                                        ),
                                        multi_line: None,
                                        quote_type: tokenizer::StringLiteralQuoteType::Double,
                                    };
                                    let token = Token::new(token_type);
                                    let empty_trivia: Vec<Token> = Vec::new();
                                    let ntokref = TokenReference::new(
                                        empty_trivia.clone(),
                                        token,
                                        empty_trivia,
                                    );
                                    *expression = ast::Expression::String(ntokref);
                                }
                            }
                            _ => (),
                        },
                        None => {
                            let token_type = TokenType::StringLiteral {
                                literal: ShortString::new(
                                    format!("[blam]\n{}", first_argument)
                                        .replace("\\", "\\\\")
                                        .replace("\"", "\\\"")
                                        .replace("\n", "\\n")
                                        .replace("\r", "\\r")
                                        .replace("\t", "\\t"),
                                ),
                                multi_line: None,
                                quote_type: tokenizer::StringLiteralQuoteType::Double,
                            };
                            let token = Token::new(token_type);
                            let empty_trivia: Vec<Token> = Vec::new();
                            let token_reference = TokenReference::new(
                                empty_trivia.clone(),
                                token,
                                empty_trivia.clone(),
                            );
                            let expression = ast::Expression::String(token_reference);
                            let trailing_trivia: Vec<Token> =
                                vec![Token::new(TokenType::Whitespace {
                                    characters: ShortString::new(" "),
                                })];
                            let comma_token = Token::new(TokenType::Symbol {
                                symbol: Symbol::Comma,
                            });
                            let token_reference =
                                TokenReference::new(empty_trivia, comma_token, trailing_trivia);
                            arguments.push(ast::punctuated::Pair::new(
                                ast::Expression::Symbol(token_reference),
                                None,
                            ));
                            arguments.push(ast::punctuated::Pair::new(expression, None));
                        }
                    }
                    new_suffixes.push(first_suffix);
                    for suffix in suffixes.skip(1) {
                        new_suffixes.push(suffix)
                    }
                    return ast::FunctionCall::new(node.prefix().clone())
                        .with_suffixes(new_suffixes);
                }
            }
        }
        node
    }
}

fn visit_directory(
    canonical_paths: &mut HashSet<PathBuf>,
    path: PathBuf,
    argument: &String,
) -> Result<(), std::io::Error> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            match entry {
                Ok(file_entry) => {
                    let entry_path = match fs::canonicalize(file_entry.path()) {
                        Ok(entry_path) => entry_path,
                        Err(error) => {
                            println!("File error on `{}`", &argument);
                            return Err(error);
                        }
                    };
                    if entry_path.is_dir() {
                        if let Err(error) = visit_directory(canonical_paths, entry_path, argument) {
                            return Err(error);
                        }
                    } else {
                        if !canonical_paths.contains(&entry_path) {
                            if let Some(extension) = entry_path.extension() {
                                if extension == "luau" || extension == "lua" {
                                    canonical_paths.insert(entry_path);
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    println!("File error on `{}`", &argument);
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() == 1 {
        println!("Blam {VERSION}");
        return Ok(());
    }

    let mut canonical_paths: HashSet<PathBuf> = HashSet::new();

    for argument in arguments.iter().skip(1) {
        if argument == "-h" || argument == "--help" {
            println!("Blam {VERSION}");
            println!("blam [files]...");
            println!("-h, --help: Print help.");
            println!("-v, --version: Print version.");
            return Ok(());
        } else if argument == "-v" || argument == "--version" {
            println!("Blam {VERSION}");
            return Ok(());
        }

        let path = match fs::canonicalize(PathBuf::from(&argument)) {
            Ok(path) => path,
            Err(error) => {
                println!("File error on `{}`", &argument);
                return Err(error);
            }
        };

        if path.is_file() {
            if !canonical_paths.contains(&path) {
                if let Some(extension) = path.extension() {
                    if extension == "luau" || extension == "lua" {
                        if canonical_paths.contains(&path) {
                        } else {
                            canonical_paths.insert(path);
                        }
                    }
                }
            }
        } else if path.is_dir() {
            if let Err(error) = visit_directory(&mut canonical_paths, path, &argument) {
                return Err(error);
            }
        }
    }

    canonical_paths.par_iter().for_each(|path| {
        let mut visitor: FunctionCallVisitor = FunctionCallVisitor::default();
        let ast = full_moon::parse(&fs::read_to_string(path).unwrap()).unwrap();
        fs::write(path, full_moon::print(&visitor.visit_ast(ast))).unwrap();
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adds_second_argument() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true)").unwrap());
        assert_eq!("assert(true, \"[blam] true\")", full_moon::print(&ast))
    }

    #[test]
    fn test_empty_single_quote_replacement() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true, '')").unwrap());
        assert_eq!("assert(true, \"[blam] true\")", full_moon::print(&ast))
    }

    #[test]
    fn test_empty_double_quote_replacement() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true, \"\")").unwrap());
        assert_eq!("assert(true, \"[blam] true\")", full_moon::print(&ast))
    }

    #[test]
    fn test_interpolated_string_replacement() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true, ``)").unwrap());
        assert_eq!("assert(true, \"[blam] true\")", full_moon::print(&ast))
    }

    #[test]
    fn test_doesnt_replace_meaningful_messages() {
        let mut visitor = FunctionCallVisitor::default();
        let ast =
            visitor.visit_ast(full_moon::parse("assert(true, \"existing message\")").unwrap());
        assert_eq!("assert(true, \"existing message\")", full_moon::print(&ast))
    }

    #[test]
    fn test_replaces_existing_blam_messages() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true, \"[blam] false\")").unwrap());
        assert_eq!("assert(true, \"[blam] true\")", full_moon::print(&ast))
    }

    #[test]
    fn test_doesnt_replace_nonstring_message() {
        let mut visitor = FunctionCallVisitor::default();
        let ast = visitor.visit_ast(full_moon::parse("assert(true, 0)").unwrap());
        assert_eq!("assert(true, 0)", full_moon::print(&ast));
    }
}
