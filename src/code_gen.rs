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
            Type: GrammarToken,
            child: Vec<Result<AstOrToken, ()>>,
        }
        impl AstNode {
            pub fn new(tok: GrammarToken, child: Vec<Result<AstOrToken, ()>>) -> AstNode {
                AstNode {
                    Type: tok,
                    child: child,
                }
            }

        }
    };

    let grammar_tokens = generate_grammar_tokens(names);

    let includes = generate_includes();

    quote! {
        #includes
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
        use std::any::Any;
        use self::GrammarToken::*; // ?
    }
}
