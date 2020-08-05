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

/// Wtf man!
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
