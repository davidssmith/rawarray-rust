//! Command line utility for manipulating `RawArray` files.


use rawarray::RawArray;
use std::env;
use std::io::{self, Read, Seek, SeekFrom};
use std::result::Result;
use std::error::Error;

fn print_usage() {
    println!("Usage:");
    println!("   ra <head|flags|eltype|elbyte|size|ndims|dims|data> file.ra");
    println!("   ra reshape file.ra dim0 dim1 dim2 ...");
    println!("RawArray file tool");
}

fn read_u64_at<IO: Read + Seek>(r: &mut IO, offset: u64) -> u64 {
    let mut buf = [0u8; 8];
    r.seek(SeekFrom::Start(offset));
    r.read_exact(&mut buf).expect("unable to read a u64");
    u64::from_le_bytes(buf)
}

fn main() -> Result<(),Box<dyn Error>> {
    if env::args().len() < 3 {
        print_usage();
    } else {
        let command = env::args().nth(1).unwrap(); 
        let filename = env::args().next().unwrap();
        let r = RawArray::valid_open(filename)?;
        match command.as_ref() {
            "head" => { }, 
            "flags" => { println!("{:x}", read_u64_at(&mut r, 8)) },
            "eltype" => { println!("{}", read_u64_at(&mut r, 16)) },
            "elbyte" => { println!("{}", read_u64_at(&mut r, 24)) },
            "size" => { println!("{}", read_u64_at(&mut r, 32)) },
            "ndims" => { println!("{}", read_u64_at(&mut r, 40)) },
            "dims" => { 
                println!("{:x}", read_u64_at(&mut r, 8)) 
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
