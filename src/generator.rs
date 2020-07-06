use proc_macro2::{ TokenStream };
use crate::tokens::{ Token, TokenTracker, get_token, token_to_string};

use quote::{ quote, format_ident };

pub struct AstNode {
    Type: Token,
    child: Option<Box<AstNode>>
}

impl AstNode {
    pub fn new(tok: Token, child: Option<Box<AstNode>>) -> AstNode {
        AstNode {
            Type: tok,
            child: child,
        }
    }

    pub fn add_child(&mut self, child: Box<AstNode>) {
        self.child = Some(child);
    }
}


pub fn generate_quote(identifier: Token, rules: Vec<Vec<Token>>) -> Result<(TokenStream, Token), String> {
    /* So, how do we generate a peg parser for the grammars we use?
     * We have a list of rules, as well as the name of the grammar we are working on.
     * 
     * For the name, we need to generate a function to match all the rules
     *  with the options they work with. For example:
     *      expr := expr '+' term
     *  will create a match_expr function, that in turn calls
     *      match_expr -> match_literal -> match_term
     *  If all of those matches succeed, then we have an expr, but if we are working
     *  with multiple matches (i.e., with a |), then we have to attempt to match the next one.
     * 
     * 
     * We also need to keep in mind how the tokens from the tokenizer will be
     * handled from the user. Additionally, we need to know how tokens are
     * represented to match terminals. To do this, a "tokenizer" will be created
     * elsewhere in the program with the following functions:
     *      self.get_token() -> Token
     *      self.mark() -> usize
     *      self.reset(pos: usize) -> void
     * This "tokenizer" should only be accessed when working with terminals in the ebnf
     * 
     * 
     * 
     */
    
    let rule_name = format_ident!("{}_matcher", token_to_string(identifier.clone())?);

    // First, we need to generate the rule matching cases
    let mut tok_rules: Vec<TokenStream> = vec![];
    for v in rules {
        //let working_rule = unwrap_rule_vector(v, identifier.clone())?;
        tok_rules.push(unwrap_rule_vector(v, identifier.clone())?);
        
    }

    // Now that we have the individual logic, we need to create the 'expect' logic for this grammar
    // So the generated code will reside inside of expect.

    let match_logic = quote!{
        fn #rule_name(tokenizer: Tokenizer) -> Result<AstNode, String> {
            #( #tok_rules )*

            Err("Parsing failed to find a match.");
        }
    };

    println!("Generated match logic: \n{}", match_logic);


    Ok((match_logic, identifier))
}

fn unwrap_rule_vector(rule: Vec<Token>, rule_name: Token) -> Result<TokenStream, String> {

    // For each element, unwrap it into its own call. 
    // Super hacky, but w/e
    // Gonna look like
    /*
        let pos = tokenizer.mark_pos()
        if match_#1 && match_#2 && ... { }

        tokenizer.reset(pos) 

    */
    let content: proc_macro2::TokenStream;
    let ast_here = make_ast_for_rule(rule.clone(), rule_name);

    content = quote!{
        let pos = tokenizer.mark_pos();
        if #( tokenizer.expect(#rule).is_okay() )&&* { // Walk off the end, see if I care!
           #ast_here 
        }
        else { tokenizer.reset(pos); }
    };

    //println!("Testing rule vector: {:#}", content);

    Ok(content)
}

fn make_ast_for_rule(rules: Vec<Token>, rule_name: Token) -> TokenStream {
    let interior = recurse_to_make_ast(rules);
    let rule_ident = format_ident!("{}", token_to_string(rule_name).unwrap());
    return quote!{
        Ok(AstNode::new(#rule_ident, Box::new(Some(#interior))))
    }
}

fn recurse_to_make_ast(rules: Vec<Token>) -> TokenStream {

    /* If we have one token, this should produce the following quote:
     * AstNode::new(token, None);
     * 
     * Multiple should produce:
     * AstNode::new(token, AstNode::new(token, None))
     * 
     * etc.
    */
    if rules.len() >= 2 {
        let mod_rules = rules[1..].to_vec(); // Drop the first item off the vector
        let top = &rules[0];
        let interior = recurse_to_make_ast(mod_rules);

        // Combine all the quotes together to form the interior tree
        return quote!{
            AstNode::new(#top, Box::new(Some(#interior)))
        }
    }
    if rules.len() == 1 { // We should be on the penultimate iteration, so don't wrap in some.
        let mod_rules = rules[1..].to_vec(); // Drop the first item off the vector
        let top = &rules[0];
        
        let interior = recurse_to_make_ast(mod_rules);

        return quote!{
            AstNode::new(#top, #interior)
        }

    }

    // Base case, i.e. no rules left in vector
    return quote!{
        None
    }

}