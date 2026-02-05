extern crate rawarray;

#[cfg(feature = "num-complex")]
use rawarray::RawArray;

#[cfg(feature = "num-complex")]
use num_complex::Complex;
#[cfg(feature = "num-complex")]
use std::fs;
#[cfg(feature = "num-complex")]
use std::io;

/// Simple example of reading and writing a RawArray and convert to
/// and from Vec<T>.
///
/// Note that the biggest change when doing this in Rust compared to
/// other languages is that you have to specify the elemental type
/// (`eltype`) in the code, so that the appropriate functions can be
/// monomorphized.
#[cfg(feature = "num-complex")]
fn main() -> io::Result<()> {
    let original = RawArray::<Complex<f32>>::read("examples/test.ra")?;
    println!("{}", original);
    original.write("tmp.ra")?;

    let reload = RawArray::<Complex<f32>>::read("tmp.ra")?;
    println!("{}", reload);
    assert_eq!(original, reload);

    let float_vec: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let from_vec: RawArray<f32> = float_vec.into();
    println!("RawArray::from(Vec):\n {}", from_vec);
    from_vec.write("tmp.ra")?;

    let into_vec: Vec<f32> = RawArray::<f32>::read("tmp.ra")?.into();
    assert_eq!(into_vec, vec![1.0, 2.0, 3.0, 4.0]);

    fs::remove_file("tmp.ra")?;

    Ok(())
}

#[cfg(not(feature = "num-complex"))]
fn main() {
    eprintln!("Error: The 'num-complex' feature is required to run this example.");
    eprintln!("Please enable it with:");
    eprintln!("  cargo run --example simple --features num-complex");
    std::process::exit(1);
}
