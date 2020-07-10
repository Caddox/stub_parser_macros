//! *************************************************************************** ///
//! File: info_collector.rs                                                     ///
//! Purpose: Defines the process for building the parser.                       ///
//! Defines: Collector                                                          ///
//!         Collector: A struct used for keeping all the information regarding  ///
//!             how the parser gets built in one place.                         ///
//! Description: This file contains the functionality of the collector, which   ///
//!     is used to build the parser using Rust's macro functionality.           ///
//!                                                                             ///
//! Beware: Thar be Tokens, Idents, and &mut tr's on these seas!                ///
//! *************************************************************************** ///

//! I see the &mut tr's in my sleep.
//!             Send help.

use proc_macro2::{ TokenStream };
use quote::{ quote, format_ident };

use crate::flat_stream::{FlatStream, Token};
use crate::token_tracker::{ TokenTracker, get_token, mark, reset, peek_as_string, get_as_string, give_max, to_string};
use crate::code_gen::generate_structures;

#[derive(Debug, Clone)]
pub struct Collector {
    pub rules: Vec<TokenStream>,
    pub terminals: Vec<TokenStream>,
    pub names: Vec<Token>,
    //flattened: FlatStream,
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
            rules, terminals, names, tracker
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
                Ok(v) => { // If okay, merge all the rules into one big rule for one vector slot.
                    let individual_rules = quote!(#(#v)* return Err(()););
                    self.rules.push(individual_rules);
                }, 
                Err(m) => {
                    let err = format!("Error generating parser: {}", m);
                    panic!(err);
                }
            }
            index += 1;
        }


        println!("======> Done generating rules.");
        //println!("======> Rules made: {:?}", self.rules.clone()[0]);

        //println!("Running boilerplate generation. . . ");
        let boilerplate = generate_structures(&self.names);
        //println!("Boilerplate generated: {:}", boilerplate);


        ///// FINAL GLUE SECTION /////
        // From here, geneate more code, append it all together, and return it out.
        let parser = self.generate_parser();
        //println!("Parser generated: {:}", parser);
        let expect = self.generate_expect_func();
        //println!("Expect func generated: {:}", expect);
        let match_f = self.generate_match_func();
        //println!("Match func generated: {:}", match_f);

        Ok(quote!{
            #boilerplate
            #parser
            #expect
            #match_f
        })
    }

    /// Working on one rule, generate the code needed for the rule to match correctly.
    fn generate_rule(&mut self, toks: Vec<Token>, index: usize) -> Result<Vec<TokenStream>, String> {
        let working_name = self.names[index].clone();
        //println!("\n\n\nGenerating rule(s) for {:}", to_string(working_name.clone())?);
        //println!("Working on {:?}", toks);

        // We have not checked for multiple rules yet, so we can loop over that.
        let mut tr = TokenTracker::new(&FlatStream::new_from_tokens(toks));
        let mut options = vec![];

        while mark(&mut tr) < give_max(&mut tr) {
            let mut current_option = vec![];

            // Loop over list until an 'or' symbol (the '|') is hit or the end of the token list is reached.
            while !peek_as_string(&mut tr).is_err()            // *.is_err needs to be first for short-circuit evaluation.
            && peek_as_string(&mut tr)? != String::from("|") {
                //println!("Looking at: {:}", peek_as_string(&mut tr)?);
                // Look for the token sequence of #( and call the requsite subroutine.
                if peek_as_string(&mut tr)? == String::from("#") {
                    let pos = mark(&mut tr);
                    let _temp = get_token(&mut tr)?; // Eat the '#' token, we dont need it.
                     // Pseudo two token lookahead


                    //// MATCH IDENTIFIERS ////
                    if peek_as_string(&mut tr)? == String::from("BEGIN") {
                        //reset(&mut tr, pos);
                        options.push(self.make_identifier_option(tr.clone(), working_name.clone())); // Make an option that was given as an identifier

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
                options.push(self.make_option(current_option, working_name.clone()));
            }

        }

        /*
        for r in options.clone() {
            println!("RULE: {:}", r);
        }
        */
        //println!("Rules produced: {:?}", options);
        Ok(options)
    }


    /// Make option will take a series of tokens that are not explicitly
    /// terminals and turn them into the if statements that are required 
    /// to generate the abstract syntax tree.
    /// 
    /// Returns a TokenStream
    fn make_option(&mut self, toks: Vec<Token>, name: Token) -> TokenStream {
        let idents: Vec<Token> = vec![];

        // This is the nested if structure needed to match a grammar.
        let ifs = self.make_if_statement(toks, idents, name, 0);

        quote!{
            let pos = mark(&mut tracker);
            #ifs
            reset(&mut tracker, pos);
        }
    
    }

    /// This function is recursive, and will populate the idents vector as 
    /// it goes down. At each iteration, the item is expected, and the matching if
    /// statement is created to match it.
    /// 
    /// When the end is reached, the AstNode constructor is built.
    fn make_if_statement(&mut self, toks: Vec<Token>, mut idents: Vec<Token>, name: Token, iteration: usize) -> TokenStream {
        if toks.len() == 0 {
            return quote!{
                return Ok(AstNode::new(#name, vec![#(#idents,)*]));
            };
        }

        // Turn the top of the list into a statement
        let (head, ident)= self.make_single_if_statement(toks[0].clone(), iteration);
        idents.push(Token::Ident(ident.clone()));

        // Recursive call.
        let body = self.make_if_statement(toks[1..].to_vec(), idents.clone(), name.clone(), iteration + 1);

        quote!{
            #head {
                #body
            }
        }
    }

    /// Making an identifier option is similar, but different.
    /// For starters, an identifier matches a tokens *.identifier field through
    /// its given type. This means that we need to grab a group of tokens, which
    /// while more convenient, means we have to change how we proceed.
    fn make_identifier_option(&mut self, mut tracker: TokenTracker, name: Token) -> TokenStream {
        // So for this to work, the contents inside of the parentheses are all going to be
        // equivalent to whatever the Token.identifier field is. For us, that is ::TokenType.
        // So if something is within the parens, it is matched and will be overloaded for later.

        // We can do this easily by grabbing the entire group marked by begin.
        while peek_as_string(&mut tracker).unwrap() != String::from("BEGIN") {
            let _null = get_token(&mut tracker);
        }

        // Return the token group that contains all the information.

        let group = get_token(&mut tracker).unwrap();

        //println!("Identifier option: {:}", quote!(#group));

        let (stmt, ident) = self.make_single_if_statement(group.clone(), 0);

        //println!("==> Identifier option: {:}", stmt);

        quote!{
            let pos = mark(&mut tracker);
            #stmt {
                return Ok(AstNode::new(#name, vec![#ident]));
            }
            reset(&mut tracker, pos);
        }
    }

    /// A function used to make a single if statement. Indent is used to make a unique
    /// identifier in the case of a grammar such as the following:
    /// 
    /// ``` expr := term '-' term ```
    /// 
    /// where the grammar could otherwise become ambiguous.
    /// 
    /// Additionally, the identifier used is returned for later use with the AstNode.
    fn make_single_if_statement(&mut self, tok: Token, indent: usize) -> (TokenStream, proc_macro2::Ident) {

        // We need this if statement because, believe it or not, _"+"_1 is not a valid token in rust.
        // Can't imagine why.
        let ident = if to_string(tok.clone()).unwrap().as_str().chars().nth(0).unwrap() == '\'' {
            format_ident!("literal_{}", indent)
        }
        else {
            format_ident!("_{}_{}", to_string(tok.clone()).unwrap(), indent)
        };


        (quote!{
            let #ident = expect(&mut tracker, &#tok);
            if #ident.is_some() 
        }, ident)

    }

    /// Function used to auto-generate the parser function, 
    /// of the following format:
    /// 
    /// Inputs: &mut TokenTracker
    /// 
    /// Outputs: Vec<AstNode>
    fn generate_parser(&self) -> TokenStream {
        let top_name = &self.names[0];
        quote!{
            pub fn parser(mut tracker: &mut TokenTracker) -> Result<Vec<AstOrToken>, ()> {
                let mut res = vec![];

                let mut tree = expect(&mut tracker, &#top_name);

                while tree.is_some() {
                    res.push(tree.unwrap());

                    tree = expect(&mut tracker, &#top_name);
                }

                Ok(res)
            }
        }
    }

    /// Helper to put all the trait definitions in one location. This 
    /// trait will be implied for three different items:
    /// 
    /// - GrammarToken
    /// - &str / maybe String
    /// - (TokenType)
    fn generate_expect_func(&self) -> TokenStream {
        quote!{
            pub fn expect(mut tracker: &mut TokenTracker, expected: &dyn Any) -> Option<AstOrToken> {
                // For each type it could be, check if the token matches. 
                if let Some(grammar) = expected.downcast_ref::<GrammarToken>() { // Ast
                    let ast = match_rule(&mut tracker, grammar);
                    match ast {
                        Ok(tree) => { return Some(AstOrToken::Ast(tree)); },
                        Err(_) => { return None }
                    }
                    //return Some(AstOrToken::Ast(match_rule(&mut tracker, grammar)));
                }
                if let Some(literal) = expected.downcast_ref::<char>() { // Token
                    // For this one, we have to match the lexeme field of the token
                    let top = get_token(&mut tracker).unwrap();
                    let lit_str = literal.to_string();
                    
                    if top.lexeme == lit_str {
                        return Some(AstOrToken::Tok(top.clone()));
                    }
                    return None;
                }
                if let Some(tok_type) = expected.downcast_ref::<TokenType> () { // Token
                    // If we get here, we expect the token.identifier to match the de-referenced type 
                    let top = get_token(&mut tracker).unwrap();
                    println!("----- In expect ------");
                    println!("Token: {:?}", top.clone());
                    println!("Identifier: {:?}", tok_type);

                    let identifier = top.clone().identifier;

                    if &identifier == tok_type {
                        return Some(AstOrToken::Tok(top));
                    }
                    else {
                        return None;
                    }
                }
                return None;
            }
        }
    }

    /// Code used to generate the function that will work in tandem with the expect
    /// function. These two will call one another until something works I guess?
    /// 
    /// Might not actually work out that way, but w/e. . . 
    fn generate_match_func(&self) -> TokenStream {
        let rules = self.rules.clone();
        let names = self.names.clone();
        
        quote!{
            fn match_rule(mut tracker: &mut TokenTracker, grammar_token: &GrammarToken) -> Result<AstNode, ()> {
                match grammar_token {
                    #( GrammarToken::#names => { #rules },)*
                    _ => {panic! ("Parsing failed to match token on grammar rule: {:?}", grammar_token) }
                }


            }
        }
    }
}

