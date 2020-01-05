use rawarray::RawArray;
use half::prelude::*;
use std::f32;
use std::fs;

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

fn main() {
    bf16();
    f16();
}
