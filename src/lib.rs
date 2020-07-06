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

mod tokens;
mod generator;
mod info_collector;
mod flat_stream;
mod token_tracker;
mod ast;

extern crate quote;

use syn::{parse_macro_input};


/// Wtf man!
#[proc_macro]
pub fn peg_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    let test = info_collector::Collector::new(input.into());
    let out = test.generate_all();
    match out {
        Ok(_) => {},
        Err(m) => {
            let err = format!("Peg gen failed with statement: {}", m);
            panic!(err);
        }
    }
    unimplemented!();
    /*
    let tokens = tokens::FlatStream::new(input.into());
    let check = tokens::parse_flat_stream(&tokens);


    match check {
        Ok(_) => {}, // Do nothing?
        Err(msg) => {
            panic!("Error while generating grammar: {}", msg);
        }
    }

    /*
    for t in tokens.tokens.clone() {
        println!("{:#?}", t);
    }
    */


    println!("{:?}", tokens.tokens[0]);

    //"fn answer() -> u32 {42}".parse().unwrap()
    proc_macro::TokenStream::new()
    
    //let input = parse_macro_input!();

    //let val = input.value();
    //println!("{:?}", val);
    

    //let _tmp = ebnf::parsing_test(input.into());



    //"fn answer() -> u32 {42}".parse().unwrap()
    */
}
