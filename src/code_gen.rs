use crate::flat_stream::Token;
/// This file contains structures that will be automatically generated
/// and added to the parser once processing has completed.
///
/// This includes items such as structure definitions, boiler-plate code, and
/// anything else that needs to be present for the parser to work at all.
///
/// This stuff is by far the most confusing, so buckle up!
use proc_macro2::TokenStream;
use quote::quote;

/// Function used to generate the structure definitions for the parser to use.
/// Included definitions:
///    
/// - AstNode
/// - GrammarToken
pub fn generate_structures(names: &Vec<Token>) -> TokenStream {
    let ast_or_token = quote! {
        #[derive(Debug, Clone)]
        pub enum AstOrToken {
            Ast(AstNode),
            Tok(Token),
        }
    };
    // Generate the abstract syntax tree info first.
    let ast_info = quote! {
        #[derive(Debug, Clone)]
        pub struct AstNode {
            pub Type: GrammarToken,
            pub child: Vec<Result<AstOrToken, ParserError>>, 
        }
        impl AstNode {
            pub fn new(tok: GrammarToken, child: Vec<Result<AstOrToken, ParserError>>) -> AstNode {
                AstNode {
                    Type: tok,
                    child: child,
                }
            }

        }
    };

    let error_type = generate_error_type();

    let grammar_tokens = generate_grammar_tokens(names);

    let includes = generate_includes();

    quote! {
        #includes
        #error_type
        #ast_info
        #ast_or_token
        #grammar_tokens
    }
}

/// Iterate over the names of the token to generate names for them all
/// in an enum-style.
fn generate_grammar_tokens(names: &Vec<Token>) -> TokenStream {
    quote! {
        #[derive(Debug, Clone)]
        pub enum GrammarToken {
            #( #names ),*
        }
    }
}

/// Small wrapper function used to move includes into the
/// function namespace.
fn generate_includes() -> TokenStream {
    quote! {
        use std::fmt;
        use std::any::Any;
        use self::GrammarToken::*; // ?
    }
}

/// Implementation of the error type for use during Parser errors.
/// 
/// Not gonna lie, I'm not sure how this is going to work out. . .
fn generate_error_type() -> TokenStream {
    quote!{
        #[derive(Debug, Clone)]
        pub struct ParserError {
            Rule: String,
            Context: Vec<Token>,
            Line: i32,
            Children: Vec<ParserError>,
        }

        impl ParserError {
            // Take the line, the rule that failed to match, and any child errors.
            fn new(mut tracker: &mut TokenTracker, rule: String, children: Vec<ParserError>) -> ParserError {
                let mut context = vec![];
                let mut line: i32 = 0;
                // Grab two tokens before and after the current token.
                // If it fails, just ignore it.
                // Two previous:
                let pos = mark(&mut tracker) as isize;
                for n in -3..=3 { // Three before, one current, three after.
                    if pos.clone() + n.clone() < 0 {continue;}
                    let index: usize = (pos.clone() + n.clone()) as usize;
                    reset(&mut tracker, index.clone());
                    let tok_check = get_token(&mut tracker);
                    if tok_check.clone().is_err() {continue;}
                    if n == 0 {
                        line = tok_check.clone().unwrap().line;
                    }
                    context.push(tok_check.unwrap());
                }
                reset(&mut tracker, pos as usize);

                ParserError {
                    Rule: rule,
                    Context: context,
                    Line: line,
                    Children: children,
                }
            }

            /// A version of format that is smaller for use in the call stack.
            fn fmt_condensed(&self) -> String {
                let mut tok_strings: Vec<String> = vec![];
                for tok in self.Context.clone() {
                    tok_strings.push(tok.lexeme);
                }
                let context = tok_strings.join(" ");

                let mut childs = vec![];
                for ch in self.Children.clone() {
                    childs.push(ch.fmt_condensed());
                }
                let children = childs.join("\t| ");

                let out = format!(
                    "{} (line {}): `{}`\n\tChildren: {}",
                    self.Rule, self.Line, context, children
                );
                out
            }
        }

        impl fmt::Display for ParserError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut tok_strs: Vec<String> = vec![];
                for tok in self.Context.clone() {
                    tok_strs.push(tok.lexeme);
                }
                let con = tok_strs.join(" ");

                let mut child = vec![];
                for item in self.Children.clone() {
                    //child.push(format!("{}", item));
                    child.push(item.fmt_condensed());
                }
                let chd = child.join("| ");

                write!(
                    f,
                    "\nParser Error: `{}`: line {} Context: `{}`\nCall stack: {}",
                    self.Rule, self.Line, con, chd
                )
            }
        }
    }
}
