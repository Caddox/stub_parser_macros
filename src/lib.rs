/*!
 * # Parser-Macros #
 * This project contains the framework and methods used to generate a PEG(ish) parser
 * for the parsing of a programming language. It allows a transformation of information from a
 * language similar to Extended BNF into a functional parser used to generate an
 * Abstract Syntax Tree.
 *
 * # Usage #
 * First, add the crate to your Cargo.toml file. Next, import the macro
 *
 * ``` use parser_macros::peg_parse; ```
 *
 * Next, you need to have created several things:
 *  
 *  - A token tracker struct, with the following function calls available to it.
 *  ```
 *  mark(&mut tracker) -> usize // A function that returns the current position of the tracker.
 *
 *  reset(&mut tracker, pos: usize) -> void // A function used to move the position of the tracker.
 *
 *  get_token(&mut tracker) -> Result<Token, String> // Function used to get the next token and advance the tracker.
 * ```
 *  - A Token struct with the following public fields:
 * ```
 *  pub lexeme: String, // For use in string matching
 *  pub identifier: TokenType // For use in literal matching.
 *  // Please note that your identifier does not have to match the name exactly;
 *  // what matters is that the field exists and the literal #() thing uses the
 *  // data structure and derives partial eq.
 * ```
 *
 * Following all this, use the macro as a top level call (i.e., not bounded to a function)
 * as follows:
 *
 * ```rust
 * peg_parse!{
 *      rule := example '-' rule "=>" #(TokenType::Identifier);
 * }
 * ```
 * Within the macro, you write rules using a form of EBNF, accepting the following
 * format:
 *
 *  ```((rule_name) ":=" (rhs) ";")*```
 *
 *  where
 *
 *  ```(rhs) := (identifier | single_char | multi-char_string | #(Explicit token identifier))*;```
 *
 * Basically you need a rule name, the ```:=```, and what the rule is, and a semicolon. The following symbols are supported:
 *
 *  - Rule_names: ```a```, ```b```, ```whatever_the_heck_you_want```
 *  - The or bar: The ```|``` symbol indicates a different option, like an or. Not having one leads to sequential matching of internals.
 *  - Groups: ```(Paren groups)```, ```[optional groups]```;
 *     - Additionally, groups can use the modifiers of: '```*```' => Match zero or more. (Just don't use the '*' with a [] -- weird shit happens.)
 *  - Token Literals: Because you can provide TokenTypes, you can match them literally through here. Ex: ```#(TokenType::FatArrow)```
 *
 * The parser will automatically attempt to match the **first rule in the list** when provided with the Token Tracker.
 *
 * When the expansion is complete, the macro exposes a function with the following signature:
 *
 * ```
 *  parser(&mut tracker: &mut TokenTracker) -> Option<AstOrToken>
 * ```
 *  
 * # Example Input #
 * ```
 * peg_parse!{
 *      language := (stmt)* #(TokenType::EOF);
 *      stmt := "let" #(TokenType::Identifier) '=' (#(TokenType::Identifier) | numeric)*;
 *      numeric := #(TokenType::Numeric);
 * }
 * ```
 *
 * # Current Bugs #
 * As hinted at above, there are some issues:
 *  - Code generation gets very confused when mixing some parenthesis with some modifiers, notably the ```*``` with square brackets.
 *  - The markers used to reset the token tracker inside of ```*``` groups can, in rare cases, overwrite one another.
 * This occurs due to the naming scheme using the length of tokens present within the paren group to form the name. In rare cases,
 * this can be equivalent. See ``` test := ((test)*)*; ```
 *  - The Rust compiler throws many warnings when expanding the compiler as each rule is expanded into a enumerated type.
 *  This is purely a cosmetic issue.
 *  - This may not actually be a full PEG parser, as pack-rat parsing has not been implemented. Left-recursion is also impossible, unless you
 * want to wait until the heat death of the universe for the parser to work :(.
 *  - The AstOrToken type is a workaround for allowing either Tokens or AstNodes as children for AstNodes. It's dumb and I hate it.
 *
 */
/// *************************************************************************** ///
/// File: token_tracker.rs                                                      ///
/// Purpose: Struct definition file.                                            ///
/// Defines: TokenTracker                                                       ///
///     TokenTracker: A structure used to walk a vector of tokens to help       ///
///         emulate the feeling of a proper PEG parser.                         ///
/// Description: This file contains functions used to help maintain accurate    ///
///     token tracking while walking the FlatStream we are given.               ///
/// *************************************************************************** ///
extern crate proc_macro;
extern crate quote;

mod code_gen;
mod flat_stream;
mod info_collector;
mod token_tracker;

/// Macro used to generate a peg(ish) parser for use in the generation of
/// abstract syntax trees.
///
/// Usage:
///
/// ```
/// peg_parse!{
///     ((rule_name) ":=" (rhs) ';')*;
///     rhs := (identifier | 'c' | "string here" | #(Explicit::Token::Type))*;
/// }
/// ```
///
/// Exposes a parser function on compilation that accepts a TokenTracker and
/// returns an Option<T>:
/// ```
/// parser(&mut tracker) -> Option<AstOrToken>
/// ```
#[proc_macro]
pub fn peg_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let test = info_collector::Collector::new(input.into());
    let out = test.generate_all();
    match out {
        Ok(m) => {
            return m.into();
        }
        Err(m) => {
            let err = format!("Peg gen failed with statement: {}", m);
            panic!(err);
        }
    }
}
