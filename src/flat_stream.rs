/// *************************************************************************** ///
/// File: flat_stream.rs                                                        ///
/// Purpose: Struct definition files.                                           ///
/// Defines: FlatStream, Token                                                  ///
///     FlatStream: A struct used to convert from a tree-based TokenStream      ///
///                 to vector.                                                  ///
///     Token: A struct used to represent individual tokens in a TokenStream    ///
/// Description: This file contains definitions needed to provide a FlatStream  ///
///     of tokens. This is more of a helper struct, as it serves as a middle    ///
///     ground between a proc_macro TokenStream and a vector of tokens.         ///
/// *************************************************************************** ///

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use quote::TokenStreamExt;

#[derive(Debug, Clone)]
pub struct FlatStream {
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub enum Token {
    Ident(Ident),
    Literal(Literal),
    Punct(Punct),
    Begin(Group, usize),
    End(Delimiter, Span),
    //Null
}

impl quote::ToTokens for Token {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Token::Ident(i) => {
                tokens.append(i.clone());
            }
            Token::Literal(l) => {
                tokens.append(l.clone());
            }
            Token::Punct(p) => {
                tokens.append(p.clone());
            }
            Token::Begin(g, _) => {
                tokens.append(g.clone());
            }
            _ => {} // Do nothing?
        }
    }
}

impl FlatStream {
    /// Generates a flattened stream of tokens from a TokenStream
    pub fn new(stream: TokenStream) -> FlatStream {
        let mut tokens = vec![];

        for tree in stream {
            flatten(&mut tokens, tree);
        }
        FlatStream { tokens }
    }

    pub fn new_from_tokens(toks: Vec<Token>) -> FlatStream {
        FlatStream {
            tokens: toks.clone(),
        }
    }
}

/// Helper function to flatten a TokenStream
fn flatten(tokens: &mut Vec<Token>, tree: TokenTree) {
    match tree {
        TokenTree::Ident(i) => tokens.push(Token::Ident(i)),
        TokenTree::Literal(l) => tokens.push(Token::Literal(l)),
        TokenTree::Punct(p) => tokens.push(Token::Punct(p)),
        TokenTree::Group(g) => {
            let start_pos = tokens.len();
            tokens.push(Token::End(g.delimiter(), g.span()));

            for tee in g.stream() {
                flatten(tokens, tee);
            }
            tokens.push(Token::End(g.delimiter(), g.span()));

            let end_pos = tokens.len();

            tokens[start_pos] = Token::Begin(g, end_pos);
        }
    }
}

/// Given a group, return the string deliminator value.
pub fn give_group_deliminator(token: Token) -> String {
    let group: proc_macro2::Group;
    match token {
        Token::Begin(g, _) => {
            group = g;
        }
        _ => {
            return String::from("Token is not of type Begin.");
        }
    }

    match group.delimiter() {
        proc_macro2::Delimiter::Parenthesis => String::from("("),
        proc_macro2::Delimiter::Brace => String::from("{"),
        proc_macro2::Delimiter::Bracket => String::from("["),
        proc_macro2::Delimiter::None => String::from("None"),
    }
}
