//! Command line utility for manipulating `RawArray` files.


use rawarray::RawArrayFile;
use std::env;
use std::result::Result;
use std::error::Error;

fn print_usage() {
    println!("Usage:");
    println!("   ra <head|flags|eltype|elbyte|size|ndims|dims|data> file.ra");
    println!("   ra reshape file.ra dim0 dim1 dim2 ...");
    println!("RawArray file tool");
}

fn main() -> Result<(),Box<dyn Error>> {
    let mut args = env::args();
    if args.len() < 3 {
        print_usage();
    } else {
        let command = args.nth(1).unwrap(); 
        let filename = args.next().unwrap();
        let mut r = RawArrayFile::valid_open(&filename)?;
        match command.as_ref() {
            "head" => { }, 
            "flags" => { println!("{:x}", r.u64_at(8)?) },
            "eltype" => { println!("{}", r.u64_at(16)?) },
            "elbyte" => { println!("{}", r.u64_at(24)?) },
            "size" => { println!("{}", r.u64_at(32)?) },
            "ndims" => { println!("{}", r.u64_at(40)?) },
            "dims" => { 
                println!("{:x}", r.u64_at(8)?) 
            },
            "data" => { },
            "reshape" => { },
            _ => { 
                print_usage();
            },
        }
    } 

    Ok(())
}
