/// File: info_collector.rs
/// Purpose: Struct definition file.
/// Defines: Some cool stuff I guess.
/// Description: I dunno yet.
/// 
/// I see the &mut tr's in my sleep.
///             Send help.

use proc_macro2::{ TokenStream };
use quote::{ quote, format_ident };

use crate::flat_stream::{FlatStream, Token};
use crate::token_tracker::{ TokenTracker, get_token, peek_token, mark, reset, peek_as_string, get_as_string, give_max, to_string};
use crate::ast::AstNode;

#[derive(Debug, Clone)]
pub struct Collector {
    pub rules: Vec<TokenStream>,
    pub terminals: Vec<TokenStream>,
    names: Vec<Token>,
    flattened: FlatStream,
    tracker: TokenTracker,
}

impl Collector {
    /// For a new collector, gather all info from a tokenstream
    pub fn new(stream: TokenStream) -> Collector {
        let rules = vec![];
        let terminals = vec![];
        let names = vec![];
        let flattened = FlatStream::new(stream);
        let tracker = TokenTracker::new(&flattened);

        Collector {
            rules, terminals, names, flattened, tracker
        }
    }

    /// Generate the parser
    pub fn generate_all(mut self) -> Result<TokenStream, String> {
        // Start by doing a pass to get each rule, deliminated by a ';'
        // Along the way, record the names for each rule for matching purposes.

        let mut mid_rules = vec![vec![]];

        while mark(&self.tracker) < give_max(&self.tracker) {
            // Assume each rule will contain the following format:
            // name ':=' (rhs)

            let name = get_token(&mut self.tracker)?;
            self.names.push(name);
            let colon_check = get_as_string(&mut self.tracker)?; // Should always be a colon
            let equ_check = get_as_string(&mut self.tracker)?; // Should always be an equals sign.

            assert_eq!(colon_check, String::from(":"), "Assignment statement was malformed");
            assert_eq!(equ_check, String::from("="), "Assignment statement was malformed");

            // Grab the rest of the rules
            let mut temp = vec![];
            while peek_as_string(&mut self.tracker)? != String::from(";") {
                temp.push(get_token(&mut self.tracker)?);
            }

            mid_rules.push(temp.clone());
            // Eat the trailing semi colon.
            let _null = get_token(&mut self.tracker)?;
        }

        // Adjust mid_rules to remove the blank list of rules
        mid_rules = mid_rules[1..].to_vec();

        // Now that each rule has been extracted, iterate over all
        // of them to generate the parser token stream.
        let mut index = 0;
        for rule in mid_rules {
            let test = self.generate_rule(rule, index);
            match test{
                Ok(_) => {}, // Do nothing!
                Err(m) => {
                    let err = format!("Error generating parser: {}", m);
                    panic!(err);
                }
            }
            index += 1;
        }





        unimplemented!();
    }

    /// Working on one rule, generate the code needed for the rule to match correctly.
    fn generate_rule(&mut self, toks: Vec<Token>, index: usize) -> Result<(), String> {
        let working_name = self.names[index].clone();
        println!("Generating rule for {:}", to_string(working_name.clone())?);
        println!("Working on {:?}", toks);

        // We have not checked for multiple rules yet, so we can loop over that.
        let mut tr = TokenTracker::new(&FlatStream::new_from_tokens(toks));
        let mut options = vec![];

        while mark(&mut tr) < give_max(&mut tr) {
            let mut current_option = vec![];

            // Loop over list until an 'or' symbol (the '|') is hit or the end of the token list is reached.
            while !peek_as_string(&mut tr).is_err()            // *.is_err needs to be first for short-circuit evaluation.
            && peek_as_string(&mut tr)? != String::from("|") {
                println!("Looking at: {:}", peek_as_string(&mut tr)?);
                // Look for the token sequence of #( and call the requsite subroutine.
                if peek_as_string(&mut tr)? == String::from("#") {
                    let pos = mark(&mut tr);
                    let temp = get_token(&mut tr)?; // Pseudo two token lookahead

                    //// MATCH IDENTIFIERS ////
                    if peek_as_string(&mut tr)? == String::from("BEGIN") {
                        //reset(&mut tr, pos);
                        options.push(make_identifier_option(tr.clone(), working_name.clone())); // Make an option that was given as an identifier

                        // There may be more identifiers in sequence after this, so skip to the end of those...
                        while peek_as_string(&mut tr)? != String::from("|") 
                        && peek_as_string(&mut tr)? != String::from("END") {
                            let _null = get_as_string(&mut tr)?;
                        }

                        // Eat the end token
                        let _null = get_token(&mut tr);
                        continue;
                    }
                    else { // Was not an identifier.
                        reset(&mut tr, pos);
                    }
                }

                current_option.push(get_token(&mut tr)?);
            }

            if !peek_as_string(&mut tr).is_err() {
                // Eat the trailing '|'
                let _null = get_token(&mut tr);
            }
        
            if current_option.len() != 0 {
                options.push(make_option(current_option, working_name.clone()));
            }

        }

        for r in options.clone() {
            println!("RULE: {:}", r);
        }
        //println!("Rules produced: {:?}", options);
        Ok(())
    }

}

/// For an identifer option, make a simpler check
fn make_identifier_option(mut tracker: TokenTracker, name: Token) -> TokenStream {
    // So for this to work, the contents inside of the parentheses are all going to be
    // equivalent to whatever the Token.identifier field is. For us, that is ::TokenType.
    // So if something is within the parens, it is matched and will be overloaded for later.

    // We can do this easily by grabbing the entire group marked by begin.
    while peek_as_string(&mut tracker).unwrap() != String::from("BEGIN") {
        let _null = get_token(&mut tracker);
    }

    // Return the token group that contains all the information.

    let group = get_token(&mut tracker).unwrap();

    println!("Identifier option: {:}", quote!(#group));

    let content = quote!{
        let pos = mark(&mut tracker);
        if expect(&mut tracker, #group) {
            AstNode::new(#name, Box::new(None));
        }
        else { reset(&mut tracker, pos); }
    };

    return content;

}

/// For each option in a grammar, generate a matching statement
fn make_option(toks: Vec<Token>, name: Token) -> TokenStream {
    let check = make_if_statement(toks.clone(), name.clone());
    let ast = make_ast(toks.clone(), name.clone());

    //println!("Make_option returned {:}", check);
    //println!(" ----- and {:}", ast);

    quote!{
        let pos = mark(&mut tracker);
        #check {
            #ast
        }
        else { reset(&mut tracker, pos); }
    }
}

/// Generate the if statement for the match option
fn make_if_statement(toks: Vec<Token>, rule_ident: Token) -> TokenStream {
    quote!{
        if #( expect(&mut tracker, #toks) )&&* // check all the statements
    }    
}

/// Generate the ASTNode to return if the match is successful.
fn make_ast(toks: Vec<Token>, name: Token) -> TokenStream {
    let content = recurse_ast(toks.clone());
    quote!{
        AstNode::new(#name, Box::new(Some(#content)))
    }
}

/// Recurse down the list of tokens, popping off the top until we reach the bottom.
fn recurse_ast(toks: Vec<Token>) -> TokenStream {
    if toks.len() == 0 {
        quote!{
            None
        }
    }
    else if toks.len() == 1 {
        // Omit the some.
        let tail = toks[1..].to_vec();
        let head = &toks[0];
    
        let content = recurse_ast(tail);

        return quote!{
            AstNode::new(#head, Box::new(#content))
        }
    }
    else {
        let tail = toks[1..].to_vec();
        let head = &toks[0];

        let content = recurse_ast(tail);

        return quote!{
            AstNode::new(#head, Box::new(Some(#content)))
        }
    }
}