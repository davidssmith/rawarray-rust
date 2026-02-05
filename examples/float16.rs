#[cfg(feature = "half")]
use half::prelude::*;
#[cfg(feature = "half")]
use rawarray::RawArray;
#[cfg(feature = "half")]
use std::f32;
#[cfg(feature = "half")]
use std::fs;

#[cfg(feature = "half")]
fn bf16() {
    let vec1: Vec<f32> = vec![f32::consts::PI, f32::consts::E, f32::consts::LN_2, 6.02e23];
    println!("f32: {:?}", vec1);
    let bvec: Vec<bf16> = vec1.iter().map(|x| bf16::from_f32(*x)).collect();
    print!("bf16: ");
    for b in &bvec {
        print!("{} ", b);
    }
    println!();
    let ra: RawArray<bf16> = bvec.clone().into();
    ra.write("bfloat16.ra").ok();

    let vec2: Vec<bf16> = RawArray::<bf16>::read("bfloat16.ra").unwrap().into();
    print!("bf16: ");
    for b in &vec2 {
        print!("{} ", b);
    }
    println!();
    fs::remove_file("bfloat16.ra").expect("unable to remove file");
    assert_eq!(bvec, vec2);
}

#[cfg(feature = "half")]
fn f16() {
    let vec1: Vec<f32> = vec![f32::consts::PI, f32::consts::E, f32::consts::LN_2, 6.02e23];
    println!("f32: {:?}", vec1);
    let bvec: Vec<f16> = vec1.iter().map(|x| f16::from_f32(*x)).collect();
    print!("f16: ");
    for b in &bvec {
        print!("{} ", b);
    }
    println!();
    let ra: RawArray<f16> = bvec.clone().into();
    ra.write("float16.ra").ok();

    let vec2: Vec<f16> = RawArray::<f16>::read("float16.ra").unwrap().into();
    print!("f16: ");
    for b in &vec2 {
        print!("{} ", b);
    }
    println!();
    fs::remove_file("float16.ra").expect("unable to remove file");
    assert_eq!(bvec, vec2);
}

#[cfg(feature = "half")]
fn main() {
    bf16();
    f16();
}

#[cfg(not(feature = "half"))]
fn main() {
    eprintln!("Error: The 'half' feature is required to run this example.");
    eprintln!("Please enable it with:");
    eprintln!("  cargo run --example float16 --features half");
    std::process::exit(1);
}
