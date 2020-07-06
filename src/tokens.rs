use proc_macro2::{ TokenStream, TokenTree, Ident, Literal, Punct, Delimiter, Group, Span, Spacing };
use quote::{ quote, format_ident, ToTokens, TokenStreamExt };
use crate::generator::generate_quote;

#[derive(Debug, Clone)]
pub struct FlatStream {
    pub tokens: Vec<Token>
}

#[derive(Debug, Clone)]
pub enum Token {
    Ident(Ident),
    Literal(Literal),
    Punct(Punct),
    Begin(Group, usize),
    End(Delimiter, Span),
    Null
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
            _ => { }, // Do nothing?
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenTracker {
    pub location: usize,
    pub max_size: usize,
    pub toks: FlatStream,
}

// constructor
impl TokenTracker {
    pub fn new(tokens: &FlatStream) -> TokenTracker {
        TokenTracker{
            location: 0,
            max_size: tokens.tokens.len(),
            toks: tokens.clone(),
        }
    }
}

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


impl FlatStream {
    pub fn new(stream: TokenStream) -> FlatStream {
        let mut tokens = vec![];

        for tree in stream {
            flatten(&mut tokens, tree);
        }

        FlatStream { tokens }
    }

}

/// Gets the next token in sequence and updates the position of the tracker
pub fn get_token(tracker: &mut TokenTracker) -> Result<Token, String> {
    let out = tracker.toks.tokens[tracker.location].clone();

    if tracker.location + 1 > tracker.max_size {
        let err = format!("Tokenizer expected more tokens than it had! Double check the grammar");
        return Err(err);
    }

    tracker.location += 1;

    Ok(out)
}

/// Returns the location that the tracker is at currently
pub fn mark(tracker: &mut TokenTracker) -> usize {
    tracker.location
}

/// Sets the tracker to a position
pub fn reset(tracker: &mut TokenTracker, pos: usize) {
    tracker.location = pos;
}

/// Returns the current token but does not iterate the counter
pub fn peek_token(tracker: &mut TokenTracker) -> Result<Token, String> {
    let current = mark(tracker);

    let out = get_token(tracker)?;

    reset(tracker, current);

    Ok(out)
}

/// If the token has a size, extract it out.
fn get_token_size(tok: Token) -> Result<usize, String> {
    match tok {
        Token::Begin(_, size) => Ok(size),
        _ => {
            let err = format!("There is no size value for the given token: {:?}", tok);
            return Err(err);
        }
    }
}

/// Formats a given token into a string value for comparison.
pub fn token_to_string(tok: Token) -> Result<String, String> {
    match tok {
        Token::Ident(i) => Ok(format!("{}", i)),
        Token::Punct(p) => Ok(format!("{}", p)),
        Token::Literal(l) => Ok(format!("{}", l)),
        Token::Begin(_, _) => Ok(format!("BEGIN")),
        Token::End(d, s) => Ok(format!("END:{:?}", s)),
        _ => {
            let err = format!("Failed to parse {:?} as a string", tok);
            return Err(err);
        }
    }
}

pub fn parse_flat_stream(tokens: &FlatStream) -> Result<(), String> {

    let tracker: &mut TokenTracker = &mut TokenTracker::new(tokens);
    let mut matcher_funcs: Vec<TokenStream> = vec![];
    let mut grammar_names: Vec<Token> = vec![];

    // First, generate all the matching code for matching tokens.
    while tracker.location < tracker.max_size {
        let (funcs, grammar_name) = parse_next(tracker)?;
        matcher_funcs.push(funcs);

        // Push the names of the grammar out too
        grammar_names.push(grammar_name);

        let c = tracker.clone();
        println!("Tracker at pos {} of {}", c.location, c.max_size);
    }

    println!("Final grammar names:\n{:?}", grammar_names);


    Ok(())
}

fn parse_next(tracker: &mut TokenTracker) -> Result<(TokenStream, Token), String> {

    // Several variables used to generate a quote
    let rule_ident = get_token(tracker)?;
    println!("Working on ident {:?} ", rule_ident);
    let mut rules: Vec<Vec<Token>> = vec![vec![]];
    //let mut rules: Vec<Vec<String>> = vec![vec![]];
    let mut rule_count = 0;

    // Check if its a "preprocessing option"
    if token_to_string(rule_ident.clone())? == String::from("#") {
        // Eat the first token to figure out what the preprocess command is
        let command = token_to_string(get_token(tracker)?)?;
        println!("===> Found command: {}", command);

        match command.as_str() {
            "FILEPATH" => {
                let filename = token_to_string(get_token(tracker)?)?;
                println!("===> Found filename: {}", filename);

                return Ok((quote!{
                    let static __filepath = #filename;
                }, Token::Null));
            }
            _ => {
                let err = format!("Unexpected preprocessing option: {}", command);
                return Err(err);
            }
        }
    }

    // Next two tokens should be ':' and '='
    let colon = get_token(tracker)?;
    let equals = get_token(tracker)?;

    assert_eq!(token_to_string(colon)?, String::from(":"));
    assert_eq!(token_to_string(equals)?, String::from("="));

    // Loop over the entire grammar... thing?
    while token_to_string(peek_token(tracker)?)? != String::from(";") {
        if token_to_string(peek_token(tracker)?)? == String::from("|") {
            rule_count += 1;
            rules.push(vec![]);
            // consume the | token
            let _null = get_token(tracker)?;
            continue;
        }
        // Match with 
        else if token_to_string(peek_token(tracker)?)? == String::from("#") {
            // Match with a token field
            println!("Found tok: {}", token_to_string(peek_token(tracker)?)?);

            // Consume the '#' token
            let mut _null = get_token(tracker)?;

            // All the tokens in the following paren grouping are
            // the item to match with the token's identifier field
            // to speed up parsing.
            println!("The following token is: {:?}", peek_token(tracker)?);

            // Pop one token?
            _null = get_token(tracker)?;
        }

        rules[rule_count].push(get_token(tracker)?);
    }

    // Consume the trailing semicolon
    let _null = get_token(tracker)?;

    /* So now that we have the term itself, and what it matches to (grammar wise),
     * we can create a quote that should be representative for this type.
     * 
     * 
     * Emphasis on """""""should"""""""
     * 
     * 
     * 
    */
    println!("{:#?}", rules);


    return generate_quote(rule_ident, rules);

}

