//! Command line utilities for manipulating `RawArray` files.
//!
//!
//!
//!


use argparse::{ArgumentParser, StoreTrue, Store};
use rawarray::RawArray;
use std::io;


fn ra_reshape(dims: &[u64]) {


}

fn usage() {


}

fn main() -> io::Result<()> {
    println!("hi");


    // reshape
    // eltype
    // elbyte
    // size
    // dims
    // ndims
    // nelem
    // data_offset
    //
   
    let mut input = String::new();
    let mut cmd = String::new();
    let mut initial_jump = 0;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("ra: RawArray file tool");
        //ap.refer(&mut verbose)
        //    .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        //ap.refer(&mut hide_keywords)
        //    .add_option(&["-k", "--no-keywords"], StoreTrue, "Hide keywords");
        //ap.refer(&mut hide_private)
        //    .add_option(&["-p", "--no-private"], StoreTrue, "Hide private tags");
        ap.refer(&mut initial_jump)
            .add_option(&["-j", "--jump"], Store, "Jump to initial position");
        ap.refer(&mut cmd)
           .add_argument("cmd", Store, "Command to issue");
        ap.refer(&mut input)
           .add_argument("input", Store, "File or directory to process");
        ap.parse_args_or_exit();
    }

    Ok(())
}
