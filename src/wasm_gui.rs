#![feature(proc_macro)]

extern crate slime_seed_finder;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[cfg(target_arch = "wasm32")]
use stdweb::js_export;

use slime_seed_finder::*;

#[cfg(target_arch = "wasm32")]
fn main(){
    // Don't start, wait for user to press button
}

#[derive(Deserialize, Debug)]
pub struct Options {
    #[serde(default)]
    chunks: String,
    #[serde(default)]
    no_chunks: String,
}

js_deserializable!( Options );

#[cfg(target_arch = "wasm32")]
#[js_export]
//pub fn slime_seed_finder(chunks_str: &str, no_chunks_str: &str) -> String {
//    let r = find_seed(chunks_str, no_chunks_str);
pub fn slime_seed_finder(o: Options) -> String {
    console!(log, "Hello from Rust");
    let r = find_seed(&o.chunks, &o.no_chunks);
    format!("Found {} seeds!\n{:#?}", r.len(), r)
}

pub fn find_seed(chunks: &str, no_chunks: &str) -> Vec<u64> {
    let c = read_chunks(chunks);
    println!("{:#?}", c);
    let nc = read_chunks(no_chunks);
    println!("{:#?}", nc);

    if let (Ok(c), Ok(nc)) = (c, nc) {
        if (c.len() == 0) && (nc.len() == 0) {
            println!("Can't find seed without chunks");
            return vec![];
        } 
        let seeds = seed_from_slime_chunks(&c, 0, &nc, 0);
        println!("Found seeds:\n{:#?}", seeds);

        {
            // Display only seeds that could be generated by java (empty box)
            let java_seeds: Vec<_> = seeds
                .iter()
                .map(|&s| Rng::extend_long_48(s))
                .collect();

            println!("Java seeds: \n{:#?}", java_seeds);
        }

        seeds
    } else {
        vec![]
    }
}
