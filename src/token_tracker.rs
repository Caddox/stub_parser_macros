/// *************************************************************************** ///
/// File: token_tracker: TokenTracker.rs                                        ///
/// Purpose: Struct definition file.                                            ///
/// Defines: TokenTracker                                                       ///
///     TokenTracker: A structure used to walk a vector of tokens to help       ///
///         emulate the feeling of a proper PEG parser.                         ///
/// Description: This file contains functions used to help maintain accurate    ///
///     token tracking while walking the FlatStream we are given.               /// 
/// *************************************************************************** ///
use crate::flat_stream::{FlatStream, Token};

#[derive(Debug, Clone)]
pub struct TokenTracker {
    location: usize,
    max_size: usize,
    toks: FlatStream,
}

impl TokenTracker {
    /// Function creates a token tracker: TokenTracker object for use later
    pub fn new(tokens: &FlatStream) -> TokenTracker {
        TokenTracker {
            location: 0,
            max_size: tokens.tokens.len(),
            toks: tokens.clone()
        }
    }
}

/// Grabs the next token, increments, and returns the token.
pub fn get_token(tracker: &mut TokenTracker) -> Result<Token, String> {
    // Check if we can grab the next token at all
    if tracker.location == tracker.max_size {
        let err = format!("TokenTracker (get_token) ran off then end of the token list (given {}, max of {})", tracker.location, tracker.max_size);
        return Err(err);
    }

    // Grab the token for output
    let out = tracker.toks.tokens[tracker.location].clone();
        
    // Increment the current position.
    tracker.location += 1;

    Ok(out)
}

/// A function for peeking at the next token in sequence without incrementing
/// the counter.
pub fn peek_token(tracker: &mut TokenTracker) -> Result<Token, String> {
    // Check if we can grab the next token at all
    if tracker.location == tracker.max_size {
        let err = format!("TokenTracker (peek_token) ran off then end of the token list (given {}, max of {})", tracker.location, tracker.max_size);
        return Err(err);
    }

    // Grab the token for output
    let out = tracker.toks.tokens[tracker.location].clone();
        
    Ok(out)
}

/// A helper function used to access the location field.
pub fn mark(tracker: &TokenTracker) -> usize {
    tracker.location
}

/// A helper to access the max size field.
pub fn give_max(tracker: &TokenTracker) -> usize {
    tracker.max_size
}

/// A helper function used to set the position of the tracker: TokenTracker
pub fn reset(tracker: &mut TokenTracker, new_location: usize) {
    tracker.location = new_location;
}

/// A helper function used to peek the next token in sequence
/// as it's string form. Helpful for checking equality.
pub fn peek_as_string(tracker: &mut TokenTracker) -> Result<String, String> {
    let peeked = peek_token(tracker)?;
    return to_string(peeked);
}

pub fn get_as_string(tracker: &mut TokenTracker) -> Result<String, String> {
    let tok = get_token(tracker)?;
    return to_string(tok);
}

/// Local function used to convert a token to a string.
pub fn to_string(tok: Token) -> Result<String, String> {
    match tok {
        Token::Ident(i) => Ok(format!("{}", i)),
        Token::Punct(p) => Ok(format!("{}", p)),
        Token::Literal(l) => Ok(format!("{}", l)),
        Token::Begin(_, _) => Ok(format!("BEGIN")),
        Token::End(_, _) => Ok(format!("END")),
        /*
        _ => {
            let err = format!("Failed to parse {:?} as a string", tok);
            return Err(err);
        }
        */
    }
}
