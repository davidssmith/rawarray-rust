//! Library for reading, writing, and manipulating RawArray files.
//!
//! A RawArray is a minimal, but complete, fast and efficient file format
//! for saving n-dimensional arrays to disk and reloading them later while
//! restoring all properties of the former arrays. RawArrays make a faster,
//! simpler substitute for MAT and HDF5 files in particular when one wants
//! to store just a single array, or when one is perfectly content to let
//! the file system handle your hierarchy of data storage, instead of
//! a bloated container format. As a bonus, RawArrays support complex
//! numbers natively, which HDF5 does not.
//!
//! The standard file extension is `.ra`, which can be pronounced
//! either "ra", as in the Egyptian god, or "are-ay", as in "array".
//! Rather than start another gif-like conflict, I think either is
//! fine.
//!
//! # Quick Start
//!
//! ```
//! use rawarray::RawArray;
//! # use std::io;
//! # fn main() -> io::Result<()> {
//! let vec1: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
//! let ra: RawArray<f32> = vec1.clone().into();
//! ra.write("myarray.ra")?;
//!
//! let vec2: Vec<f32> = RawArray::<f32>::read("myarray.ra")?.into();
//! assert_eq!(vec1, vec2);
//! # Ok(())
//! # }
//! ```
//!

#![deny(warnings, missing_docs)]

use half::prelude::*;
use ndarray::{Array, Array1, ArrayD};
use num_complex::Complex;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::{fmt, mem, slice};

const FLAG_BIG_ENDIAN: u64 = 1;
const FLAG_ENCODED: u64 = 2; // run-length encoding for Ints
const FLAG_BITS: u64 = 4; // array element is a single bit
const ALL_KNOWN_FLAGS: u64 = FLAG_BIG_ENDIAN | FLAG_ENCODED | FLAG_BITS;
// TODO: see if reading > 2 GB is a problem in Rust
//const MAX_BYTES       : u64 = 1<<31;
//
//const MAGIC_NUMBER    : u64 = 0x79_61_72_72_61_77_61_72;
const MAGIC_NUMBER: u64 = 0x79_61_72_72_61_77_61_72u64;
//6172 6177 7272 7961

/*
enum ElementType {
    User = 0,
    Int,
    UInt,
    Float,
    Complex,
}
*/

/// Helper trait to constrain to elemental types that make sense.
// ```rust
// assert_eq!(i8::ra_type_code(), 1);
// assert_eq!(i8::ra_elbyte(), 1);
// ``
pub trait RawArrayType: Clone + Copy + Debug + Display + Send + Sync {
    /// Integer type code representing class of element type:
    ///
    /// 0. user defined
    /// 1. signed integer
    /// 2. unsigned integer
    /// 3. IEEE floating point
    /// 4. complex
    /// 5. brain floating point (bfloat16)
    ///
    /// 6 and higher are reserved for future use, like maybe
    /// Unicode or SIMD types
    ///
    /// The default type code is 0, because it puts the burden
    /// on the user to deal with unknown types, hopefully through
    /// a pull request to this repo!
    /// ```
    /// use num_complex::Complex;
    /// use rawarray::{RawArray, RawArrayType};
    /// assert_eq!(i8::ra_type_code(), 1);
    /// assert_eq!(u8::ra_type_code(), 2);
    /// assert_eq!(f32::ra_type_code(), 3);
    /// assert_eq!(Complex::<f32>::ra_type_code(), 4);
    /// ```
    fn ra_type_code() -> u64 {
        0
    }
}

impl RawArrayType for i8 {
    fn ra_type_code() -> u64 {
        1
    }
}
impl RawArrayType for i16 {
    fn ra_type_code() -> u64 {
        1
    }
}
impl RawArrayType for i32 {
    fn ra_type_code() -> u64 {
        1
    }
}
impl RawArrayType for i64 {
    fn ra_type_code() -> u64 {
        1
    }
}
impl RawArrayType for i128 {
    fn ra_type_code() -> u64 {
        1
    }
}
impl RawArrayType for u8 {
    fn ra_type_code() -> u64 {
        2
    }
}
impl RawArrayType for u16 {
    fn ra_type_code() -> u64 {
        2
    }
}
impl RawArrayType for u32 {
    fn ra_type_code() -> u64 {
        2
    }
}
impl RawArrayType for u64 {
    fn ra_type_code() -> u64 {
        2
    }
}
impl RawArrayType for u128 {
    fn ra_type_code() -> u64 {
        2
    }
}
impl RawArrayType for f32 {
    fn ra_type_code() -> u64 {
        3
    }
}
impl RawArrayType for f64 {
    fn ra_type_code() -> u64 {
        3
    }
}
impl RawArrayType for Complex<f32> {
    fn ra_type_code() -> u64 {
        4
    }
}
impl RawArrayType for Complex<f64> {
    fn ra_type_code() -> u64 {
        4
    }
}
impl RawArrayType for bf16 {
    fn ra_type_code() -> u64 {
        5
    }
}
impl RawArrayType for f16 {
    fn ra_type_code() -> u64 {
        3
    }
}

/// Combine the two necessary traits for efficient file parsing
trait RawArrayIO: BufRead + Seek {}

/// Wraps reading for some simpler parsing code
pub struct RawArrayFile(Box<dyn RawArrayIO>);

impl<T: Read + Seek> RawArrayIO for BufReader<T> {}

impl RawArrayFile {
    /// Open and validate a `RawArray` file and return a `File` handle,
    /// but don't attempt to parse.
    pub fn valid_open<P: AsRef<Path>>(path: P) -> io::Result<RawArrayFile> {
        let f = File::open(path)?;
        let r = BufReader::new(f);
        let mut raf = RawArrayFile(Box::new(r));
        let magic = raf.u64_at(0)?;
        if magic != MAGIC_NUMBER {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid magic, likely not a RawArray file.",
            ));
        }
        Ok(raf)
    }

    /// Return next `u64` in the stream
    pub fn u64(&mut self) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.0.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Seek to given position in a RawArrayFile
    pub fn seek(&mut self, loc: u64) -> io::Result<()> {
        self.0.seek(SeekFrom::Current(loc as i64))?;
        Ok(())
    }

    /// Return a `u64` located at an offset within the file
    /// without affecting current reading location
    pub fn u64_at(&mut self, offset: u64) -> io::Result<u64> {
        let cur_loc = self.0.seek(SeekFrom::Current(0))?;
        self.0.seek(SeekFrom::Start(offset))?;
        let mut buf = [0u8; 8];
        self.0.read_exact(&mut buf)?;
        self.0.seek(SeekFrom::Start(cur_loc))?;
        Ok(u64::from_le_bytes(buf))
    }
}

/// Container type for RawArrays
#[derive(Clone, Debug, PartialEq)]
pub struct RawArray<T: RawArrayType> {
    flags: u64,
    eltype: u64,
    elbyte: u64,
    size: u64,
    ndims: u64,
    dims: Vec<u64>,
    data: Vec<T>,
}

/*
 * Some private helper functions for data type conversion
 * and binary reading
 */

fn read_u64<T: Read>(r: &mut T) -> u64 {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).expect("unable to read a u64");
    u64::from_le_bytes(buf)
}

fn write_u64<T: Write>(r: &mut T, n: u64) -> io::Result<()> {
    r.write_all(&n.to_le_bytes())?;
    Ok(())
}

fn from_u8<T: RawArrayType>(v: Vec<u8>) -> Vec<T> {
    let data = v.as_ptr();
    let len = v.len();
    let capacity = v.capacity();
    let element_size = mem::size_of::<T>();

    // Make sure we have a proper amount of capacity (may be overkill)
    assert_eq!(capacity % element_size, 0);
    // Make sure we are going to read a full chunk of stuff
    assert_eq!(len % element_size, 0);

    unsafe {
        // Don't allow the current vector to be dropped
        // (which would invalidate the memory)
        mem::forget(v);

        Vec::from_raw_parts(data as *mut T, len / element_size, capacity / element_size)
    }
}

fn as_u8_slice<T: RawArrayType>(v: &[T]) -> &[u8] {
    let element_size = mem::size_of::<T>();
    unsafe { slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * element_size) }
}

fn flags_as_string(flags: u64) -> String {
    let mut s = String::new();
    if flags & FLAG_BIG_ENDIAN != 0 {
        s.push_str("BigEndian ");
    } else {
        s.push_str("LittleEndian ");
    }
    if flags & FLAG_ENCODED != 0 {
        s.push_str("RLE ");
    }
    if flags & FLAG_BITS != 0 {
        s.push_str("BitArray");
    }
    s
}

impl<T: RawArrayType> Default for RawArray<T> {
    fn default() -> Self {
        RawArray {
            flags: 0,
            eltype: T::ra_type_code(),
            elbyte: mem::size_of::<T>() as u64,
            size: 0,
            ndims: 0,
            dims: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<T: RawArrayType> From<Vec<T>> for RawArray<T> {
    /// Create a `RawArray<T>` from a `Vec<T>`
    fn from(v: Vec<T>) -> RawArray<T> {
        let size: u64 = (v.len() * mem::size_of::<T>()) as u64;
        let eltype = T::ra_type_code();
        let elbyte = mem::size_of::<T>() as u64;
        let dims = vec![v.len() as u64];
        RawArray {
            flags: 0,
            eltype,
            elbyte,
            size,
            ndims: 1,
            dims,
            data: v,
        }
    }
}

impl<T: RawArrayType> From<ArrayD<T>> for RawArray<T> {
    /// Create a `RawArray<T>` from an `ArrayD<T>`
    fn from(a: ArrayD<T>) -> RawArray<T> {
        a.into_raw_vec_and_offset().0.into()
    }
}

impl<T: RawArrayType> Into<Vec<T>> for RawArray<T> {
    /// Create a `Vec<T>` from a `RawArray<T>`
    fn into(self) -> Vec<T> {
        self.data
    }
}

impl<T: RawArrayType> Into<Array1<T>> for RawArray<T> {
    /// Create a `Vec<T>` from a `RawArray<T>`
    fn into(self) -> Array1<T> {
        Array::from(self.data)
    }
}

impl<T: RawArrayType> Display for RawArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "flags: {}", flags_as_string(self.flags))?;
        writeln!(f, "eltype: {}", self.eltype)?;
        writeln!(f, "elbyte: {}", self.elbyte)?;
        writeln!(f, "size: {}", self.size)?;
        writeln!(f, "ndims: {}", self.ndims)?;
        writeln!(f, "dims: {:?}", self.dims)?;
        write!(f, "data: {:?}", self.data)
    }
}

impl<T: RawArrayType> RawArray<T> {
    /// Create a new `RawArray<T>` using default values.
    pub fn new() -> RawArray<T> {
        RawArray::default()
    }

    /// Create a new `RawArray<T>` with same type and dimensions but new data
    pub fn clone_with_data(&self, data: Vec<T>) -> RawArray<T> {
        RawArray {
            flags: self.flags,
            eltype: self.eltype,
            elbyte: self.elbyte,
            size: self.size,
            ndims: self.ndims,
            dims: self.dims.clone(),
            data,
        }
    }

    /// Boolean feature flags, endianness, etc.
    pub fn flags(&self) -> u64 {
        self.flags
    }
    /// Elemental type code.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<u8> = vec![0xc0, 0xff, 0xee].into();
    /// assert_eq!(r.eltype(), 2);
    /// ```
    pub fn eltype(&self) -> u64 {
        self.eltype
    }
    /// Size of each individual element of the array in bytes.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<u64> = vec![3, 1, 4].into();
    /// assert_eq!(r.elbyte(), 8);
    /// ```
    pub fn elbyte(&self) -> u64 {
        self.elbyte
    }
    /// Total size of array data in bytes.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<u32> = vec![8, 6, 7, 5, 3, 0, 9].into();
    /// assert_eq!(r.size(), 28);
    /// ```
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Number of dimensions of array.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<i32> = vec![1, 1, 2, 3, 5, 8].into();
    /// assert_eq!(r.ndims(), 1);
    /// ```
    pub fn ndims(&self) -> u64 {
        self.ndims
    }
    /// *Copy* of the array dimensions.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<i16> = vec![1, 0, 1, 0].into();
    /// assert_eq!(r.dims(), vec![4]);
    /// ```
    pub fn dims(&self) -> Vec<u64> {
        self.dims.clone()
    }
    /// *Copy* of the array data.
    /// ```
    /// # use rawarray::RawArray;
    /// let v: Vec<f32> = vec![3.14, 2.72, 1.618, 1.414];
    /// let r: RawArray<f32> = v.clone().into();
    /// assert_eq!(r.data(), v);
    /// ```
    pub fn data(&self) -> Vec<T> {
        self.data.clone()
    }
    /// Get a reference to the dims vector.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<u16> = vec![1, 0, 1, 0].into();
    /// assert_eq!(*r.dims_as_ref(), vec![4]);
    /// ```
    pub fn dims_as_ref(&self) -> &Vec<u64> {
        &self.dims
    }

    /// Get a reference to the data vector.
    /// ```
    /// # use rawarray::RawArray;
    /// let r: RawArray<u16> = vec![1, 0, 1, 0].into();
    /// assert_eq!(*r.data_as_ref(), vec![1, 0, 1, 0]);
    /// ```
    pub fn data_as_ref(&self) -> &Vec<T> {
        &self.data
    }

    /// Reshape to new dimensions
    /// ```
    /// # use rawarray::RawArray;
    /// let mut r: RawArray<u16> = vec![1, 0, 1, 0].into();
    /// r.reshape([2, 2].to_vec());
    /// assert_eq!(r.dims(), vec![2, 2]);
    /// ```
    pub fn reshape(&mut self, new_dims: Vec<u64>) {
        let new_nelem: u64 = new_dims.iter().product();
        let old_nelem: u64 = self.dims.iter().product();
        assert_eq!(new_nelem, old_nelem);
        self.dims = new_dims;
    }

    /// Read the file header
    fn read_header<R: Read>(&mut self, mut r: &mut R) -> io::Result<()> {
        // read header, which should always be LittleEndian
        let magic = read_u64(&mut r);
        assert_eq!(magic, MAGIC_NUMBER);

        self.flags = read_u64(&mut r);
        if self.flags & ALL_KNOWN_FLAGS != 0 {
            panic!(
                "Unknown flags encounter in header. This file must have been written
                    with a newer version of the library. Please upgrade your RawArray
                    installation by running `cargo update`."
            );
        }
        self.eltype = read_u64(&mut r);
        assert_eq!(self.eltype, T::ra_type_code());
        self.elbyte = read_u64(&mut r);
        assert_eq!(self.elbyte, mem::size_of::<T>() as u64);
        self.size = read_u64(&mut r);
        self.ndims = read_u64(&mut r);

        // read dimensions
        //let mut dims: Vec<u64> = Vec::with_capacity(ndims as usize);
        self.dims.reserve(self.ndims as usize);
        for _ in 0..self.ndims {
            self.dims.push(read_u64(&mut r));
        }
        let nelem: u64 = self.dims.iter().product(); //fold(1, |acc, x| acc * x);
        assert_eq!(nelem * self.elbyte, self.size);
        Ok(())
    }

    /// Read the data section
    fn read_data<R: Read>(&mut self, r: &mut R) -> io::Result<()> {
        let mut byte_data: Vec<u8> = Vec::with_capacity(self.size as usize);
        let bytes_read = r.read_to_end(&mut byte_data)? as u64;
        assert_eq!(bytes_read, self.size);
        self.data = from_u8::<T>(byte_data);
        Ok(())
    }

    /// Read a `RawArray<T>` from a file.
    /// ```
    /// # use std::io;
    /// use rawarray::RawArray;
    /// # fn main() -> io::Result<()>{
    /// # let vec1: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    /// # let ra: RawArray<f32> = vec1.clone().into();
    /// # ra.write("myarray.ra")?;
    /// let ra = RawArray::<f32>::read("myarray.ra")?;
    /// // Or Into<Vec<T>> makes it easy to read directly into a Vec
    /// let vec: Vec<f32> = RawArray::<f32>::read("myarray.ra")?.into();
    /// # assert_eq!(vec1, vec);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<RawArray<T>> {
        let f = File::open(path)?;
        let mut r = BufReader::new(f);
        let mut ra = RawArray::default();
        ra.read_header(&mut r)?;
        ra.read_data(&mut r)?;
        Ok(ra)
    }

    fn write_header<W: Write>(&self, mut w: &mut W) -> io::Result<()> {
        write_u64(&mut w, MAGIC_NUMBER)?;
        write_u64(&mut w, self.flags)?;
        write_u64(&mut w, self.eltype)?;
        write_u64(&mut w, self.elbyte)?;
        write_u64(&mut w, self.size)?;
        write_u64(&mut w, self.ndims)?;
        for d in self.dims.iter() {
            write_u64(&mut w, *d)?;
        }
        Ok(())
    }

    fn write_data<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(as_u8_slice(&self.data))?;
        Ok(())
    }

    /// Write a `RawArray<T>` to file.
    /// ```
    /// # use std::io;
    /// use rawarray::RawArray;
    /// # fn main() -> io::Result<()>{
    /// let ra: RawArray<f32> = vec![1.0, 2.0, 3.0, 4.0].into();
    /// ra.write("myarray.ra")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let f = File::create(path)?;
        let mut w = BufWriter::new(f);
        self.write_header(&mut w)?;
        self.write_data(&mut w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bf16() {
        use super::*;
        use std::fs;
        let vec1: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let bvec: Vec<bf16> = vec1.iter().map(|x| bf16::from_f32(*x)).collect();
        let ra: RawArray<bf16> = bvec.clone().into();
        ra.write("test_bf16.ra").ok();
        let vec2: Vec<bf16> = RawArray::<bf16>::read("test_bf16.ra").unwrap().into();
        fs::remove_file("test_bf16.ra").expect("unable to remove file");

        assert_eq!(bvec, vec2);
    }
    #[test]
    fn f16() {
        use super::*;
        use std::fs;
        let vec1: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let bvec: Vec<f16> = vec1.iter().map(|x| f16::from_f32(*x)).collect();
        let ra: RawArray<f16> = bvec.clone().into();
        ra.write("test_f16.ra").ok();
        let vec2: Vec<f16> = RawArray::<f16>::read("test_f16.ra").unwrap().into();
        fs::remove_file("test_f16.ra").expect("unable to remove file");

        assert_eq!(bvec, vec2);
    }
}
