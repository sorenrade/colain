//! Parser for the Common Layer Interface (.cli) file format
//!
//! The parser is written according to the spec provided [`here.`]
//!
//! This library works by examining the data in place and collecting pointers to the geometry sections. The CLI object consists of a [`Vec`] of layers. Each layer in turn contains
//! a [`Vec`] of loops and hatches respectively.
//!
//! **Note:** In keeping with the performance oriented nature of the library, conversions to real units using the UNITS portion of the header file is not done automatically.
//! Remember to perform the conversion if necessary.
//!
//! **Note:** This library does not yet support parsing of ASCII formated files. Nor has it been tested extensively since .cli files are hard to come by.
//! Please feel free to submit bug reports or .cli files for testing.
//!
//! [`here.`]: https://www.hmilch.net/downloads/cli_format.html

//! # Examples

//! ## Loading and parsing a file
//! ```
//! use std::fs::File;
//! use std::io::prelude::*;
//! use colain::{
//!		CLI,
//!		clitype::{LongCLI, ShortCLI}
//!	};
//!
//! let mut buf: Vec<u8> = Vec::new();
//! File::open("example.cli").unwrap().read_to_end(&mut buf).unwrap();
//!
//! let model = CLI::<LongCLI>::new(&buf).unwrap();
//!
//! println!("{:?}", model.header());
//! ```

//! ## Iterating on each point of each loop in each layer
//! See above for how to initialize model
//!```
//! use colain::Point; // import the Point trait to provide access via .x() and .y()
//! for layer in model.iter() {
//!     for a_loop in layer.iter_loops() {
//!         for point in a_loop.iter() {
//!             let x = point.x();
//!             let y = point.y();
//!         }
//!     }
//! }
//!```

use bytes::Buf;
use clitype::*;
use std::fmt::Debug;
use std::mem::size_of;

mod util;
use util::*;

/// A [`CLIType`] must be specified when creating a [`CLI`] object.
///
/// The CLI spec dictates that two different binary formats to express geometry data:
/// - Short: coordinates are stored as [`u16`]
/// - Long: coordinates are stored as [`f32`]
///
/// While it is not explicitly required that a file consists of only one type of geometry,
/// this library requires that the entire file consists of just one variation. If the variation
/// that was specified does not match the file an [`Err`] containing the signal [`Error::TypeMismatch`]
/// will be returned.
pub mod clitype {
    use super::*;

    /// A type of CLI file
    pub trait CLIType
    where
        Self::Meta: Debug + Copy,
        Self::Coord: Debug + Copy,
    {
        /// Primitive type used to store metadata such as id, direction, etc.
        ///
        /// The CLI will be of either type [`ShortCLI`] or [`LongCLI`].
        /// See the documentation for these items to know what the primitive will be.
        type Meta;
        /// Primitive type used to store coordinates
        ///
        /// The CLI will be of either type [`ShortCLI`] or [`LongCLI`].
        /// See the documentation for these items to know what the primitive will be.
        type Coord;

        // Command used to indicate a new layer
        #[doc(hidden)]
        const CMD_LAYER: u16;
        // Command used to indicate a new polyline
        #[doc(hidden)]
        const CMD_PLINE: u16;
        // Command used to indicate a new set of hatches
        #[doc(hidden)]
        const CMD_HATCH: u16;

        // Pop a metadata from the buffer
        #[doc(hidden)]
        fn get_meta(buf: &mut &[u8], aligned: bool) -> Self::Meta;
        // Pop a coordinate from the buffer
        #[doc(hidden)]
        fn get_coord(buf: &mut &[u8], aligned: bool) -> Self::Coord;
        // Pop a metadata from the buffer and cast to a usize
        #[doc(hidden)]
        fn get_usize(buf: &mut &[u8], aligned: bool) -> usize;
    }

    /// Configures the parser to use the short version of the CLI spec.
    ///
    /// In this version coordinates are stored as [`u16`] and metadata
    /// (ID, direction, etc.) are stored as [`u16`].
    #[derive(Debug)]
    pub struct ShortCLI();
    /// Configures the parser to use the long version of the CLI spec.
    ///
    /// In this version coordinates are stored as [`f32`] and metadata
    /// (ID, direction, etc.) are stored as [`i32`].
    #[derive(Debug)]
    pub struct LongCLI();

    impl CLIType for ShortCLI {
        type Meta = u16;
        type Coord = u16;
        const CMD_LAYER: u16 = 128;
        const CMD_PLINE: u16 = 129;
        const CMD_HATCH: u16 = 131;
        fn get_meta(buf: &mut &[u8], aligned: bool) -> Self::Meta {
            let t = buf.get_u16_le();
            if aligned {
                buf.advance(2)
            };
            return t;
        }
        fn get_coord(buf: &mut &[u8], aligned: bool) -> Self::Coord {
            let t = buf.get_u16_le();
            if aligned {
                buf.advance(2)
            };
            return t;
        }
        fn get_usize(buf: &mut &[u8], aligned: bool) -> usize {
            let t = buf.get_u16_le() as usize;
            if aligned {
                buf.advance(2)
            };
            return t;
        }
    }

    impl CLIType for LongCLI {
        type Meta = i32;
        type Coord = f32;
        const CMD_LAYER: u16 = 127;
        const CMD_PLINE: u16 = 130;
        const CMD_HATCH: u16 = 132;
        fn get_meta(buf: &mut &[u8], _aligned: bool) -> Self::Meta {
            buf.get_i32_le()
        }
        fn get_coord(buf: &mut &[u8], _aligned: bool) -> Self::Coord {
            buf.get_f32_le()
        }
        fn get_usize(buf: &mut &[u8], _aligned: bool) -> usize {
            buf.get_i32_le() as usize
        }
    }
}

/// Reinterpret [T; 2] as a point
pub trait Point<T: Copy> {
    /// Get the x component of the point
    fn x(&self) -> T;
    /// Get the y component of the point
    fn y(&self) -> T;
}
impl<T: Copy> Point<T> for [T; 2] {
    #[inline]
    fn x(&self) -> T {
        self[0]
    }
    #[inline]
    fn y(&self) -> T {
        self[1]
    }
}

/// Reinterpret [T; 4] as two points
/// ```
/// use std::fs::File;
/// use std::io::prelude::*;
/// use colain::{
///		CLI, Segment, Point,
///		clitype::*
///	};
///
/// let mut buf: Vec<u8> = Vec::new();
/// File::open("example.cli").unwrap().read_to_end(&mut buf).unwrap();
///
/// let model = CLI::<LongCLI>::new(&buf).unwrap();
///	let x: f32 = model.iter().next().unwrap() // first layer
///			.iter_hatches().next().unwrap() // first set of hatches in layer
///			.iter().next().unwrap() // first segment in hatches
///			.start() // first point in segment
///			.x(); // x value of first point in segment
///
/// ```
pub trait Segment<T: Copy> {
    /// Get the first point
    fn start(&self) -> [T; 2];
    /// Get the second point
    fn end(&self) -> [T; 2];
}
impl<T: Copy> Segment<T> for [T; 4] {
    #[inline]
    fn start(&self) -> [T; 2] {
        // SAFETY: By nature of impl constrained to arrays of len 4
        unsafe { *(&self[0..=1] as *const [T] as *const [T; 2]) }
    }
    #[inline]
    fn end(&self) -> [T; 2] {
        // SAFETY: By nature of impl constrained to arrays of len 4
        unsafe { *(&self[2..=3] as *const [T] as *const [T; 2]) }
    }
}

/// Object representing a loop inside of a [`Layer`]
///
/// Each [`Loop`] contains an id (see the spec for uses), a direction and a slice pointer to the geometry data.
///
/// According to the spec, the direction could be one of 3 values. However, it is left as an integer since some slicers interpret
/// these values differently.
/// According to the spec the direction can be:
/// - 0 : clockwise (internal)
/// - 1 : counter-clockwise (external)
/// - 2 : open line (no solid)
///
/// Each point is stored as an array of length two of the [`CLIType`]'s associated Coord type.
/// The [`Point`] trait is provided as a more elegant way to access the data.
#[derive(Debug, Clone)]
pub struct Loop<'a, T: CLIType> {
    id: <T as CLIType>::Meta,
    dir: <T as CLIType>::Meta,
    points: &'a [<T as CLIType>::Coord],
}

impl<'a, T: CLIType> Loop<'a, T> {
    /// Iterate over each point in the loop as [T; 2]
    ///
    /// Note availability of [`Point`] trait for a cleaner interface
    pub fn iter(&'a self) -> ArrayChunksCopy<'a, <T as CLIType>::Coord, 2> {
        ArrayChunksCopy::<'_, <T as CLIType>::Coord, 2>::new(self.points)
    }
    /// Get the CLI ID of this primitive
    pub fn id(&self) -> <T as CLIType>::Meta {
        self.id
    }
    /// Get the direction of this loop
    pub fn dir(&self) -> <T as CLIType>::Meta {
        self.dir
    }
    /// Pointer into the segment of the file that contains this geometry
    pub fn points(&'a self) -> &'a [<T as CLIType>::Coord] {
        self.points
    }
}

/// Collection of hatches inside a [`Layer`]
///
/// Each hatch is a line segment with a start and end point
/// the [`Segment`] trait is provided as an abstraction layer over the
/// internal storage of each segment which is [T; 4]
#[derive(Debug, Clone)]
pub struct Hatches<'a, T: CLIType> {
    id: <T as CLIType>::Meta,
    points: &'a [<T as CLIType>::Coord],
}

impl<'a, T: CLIType> Hatches<'a, T> {
    /// Iterate over hatches as segments
    ///
    /// Note availability of [`Segment`] trait for a cleaner interface
    pub fn iter(&'a self) -> ArrayChunks<'a, <T as CLIType>::Coord, 4> {
        ArrayChunks::<'_, <T as CLIType>::Coord, 4>::new(self.points)
    }
    /// Get the CLI ID of this primitive
    pub fn id(&self) -> <T as CLIType>::Meta {
        self.id
    }
    /// Pointer into the segment of the file that contains this geometry.
    /// The array should consist of sets of 2 points where each point
    /// consists of an X element then a Y element.
    pub fn points(&'a self) -> &'a [<T as CLIType>::Coord] {
        self.points
    }
}

/// Represents a layer of a 3D object
///
///
#[derive(Debug, Clone)]
pub struct Layer<'a, T: CLIType> {
    height: <T as CLIType>::Coord,
    loops: Vec<Loop<'a, T>>,
    hatches: Vec<Hatches<'a, T>>,
}
impl<'a, T: CLIType> Layer<'a, T> {
    /// Iterator over each loop in the layer
    pub fn iter_loops(&'a self) -> std::slice::Iter<'a, Loop<'a, T>> {
        self.loops.iter()
    }
    /// Iterator over each set of hatches in the layer
    pub fn iter_hatches(&'a self) -> std::slice::Iter<'a, Hatches<'a, T>> {
        self.hatches.iter()
    }
    /// Get the height of the layer relative to the bottom of the part.
    /// Note that layer thickness is not encoded in the CLI format, it must be
    /// calculated from the height delta between two slices.
    pub fn height(&self) -> <T as CLIType>::Coord {
        self.height
    }
}

/// Contains all available CLI header information
#[derive(Debug, Clone)]
pub struct Header {
    /// True if the CLI file stores data in a binary format
    pub binary: bool,
    /// How many millimeters each coordinate unit represents
    pub units: f64,
    /// CLI version
    pub version: f32,
    /// True if the binary file is aligned
    pub aligned: bool,
    /// The header can optionally declare the number of layers in the file`
    pub layers: Option<usize>,
}

/// Errors encountered when parsing a CLI file
#[derive(Debug)]
pub enum Error {
    /// File is too short to contain a CLI file.
    EmptyFile,
    /// File does not contain a header section.
    NoHeader,
    /// Header does not contain valid UTF-8.
    HeaderInvalidUTF8,
    /// The header indicates that this file contains an ASCII encoded geometry section.
    /// This library does not support his format at this time.
    UnsupportedGeometryFormat,
    /// Header is missing a required element:
    /// - 0: Indication of binary or ASCII geometry section
    /// - 1: Units
    /// - 2: Version
    HeaderIncomplete(u8),
    /// A numeric header value could not be parsed.
    InvalidHeaderValue,
    /// One of 6 binary commands was expected in the next two bytes, instead, this value was found.
    /// Most likely the file is corrupted. It is possible the file contains commands not included in the CLI spec.
    ///
    /// A bug in this library may also be present. Please consider submitting the .cli file in a PR. Thank you.
    InvalidGeometryCommand(u16),
    /// The file in invalid because it has geometry elements in the geometry section before specifying the first layer.
    ElementOutsideLayer,
    /// An element in the geometry section indicated that more data was present but the EOF has been reached.
    UnexpectedEOF,
    /// The [`CLIType`] specified when declaring the [`CLI`] parser does not match the data in the geometry section of the file.
    TypeMismatch,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}

/// Light abstraction over a CLI file
pub struct CLI<'a, T: CLIType> {
    // raw: &'a Vec<u8>,
    header: Header,
    layers: Vec<Layer<'a, T>>,
}

impl<'a, T: CLIType> CLI<'a, T> {
    /// Takes a buffer containing the .cli file
    /// and finds all the offsets to each geometry section.
    ///
    /// See crate level documentation for usage.
    pub fn new(raw: &'a [u8]) -> Result<Self, Error> {
        let (mut gstart, header) = CLI::<T>::parse_header(&raw)?;
        if !header.binary {
            Err(Error::UnsupportedGeometryFormat)?;
        }

        if header.aligned {
            gstart = 4 * ((gstart - 1) / 4) + 4;
        }
        let mut geom = &raw[gstart..];

        let mut this = CLI {
            header,
            layers: Vec::new(),
        };

        let mut current_layer = None;
        while this.next_element(&mut current_layer, &mut geom)? {}
        Ok(this)
    }

    /// Get file metadata
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline]
    fn parse_header(raw: &[u8]) -> Result<(usize, Header), Error> {
        // TODO: UTF-8 aware audit
        let pattern: &[u8] = b"$$HEADEREND";
        if raw.len() <= pattern.len() {
            Err(Error::EmptyFile)?;
        }

        let mut search_index = 0;
        let mut pattern_index = 0;

        // TODO: use windows iterator here
        while search_index < raw.len() && pattern_index < pattern.len() {
            if raw[search_index] == pattern[pattern_index] {
                pattern_index += 1;
            } else {
                pattern_index = 0;
            }
            search_index += 1;
        }

        if pattern_index < pattern.len() {
            Err(Error::NoHeader)?;
        }

        let header =
            std::str::from_utf8(&raw[0..search_index]).map_err(|_| Error::HeaderInvalidUTF8)?;

        // Format(binary, ascii), units, version, date, dimension, layers, align
        let mut items: [Option<&str>; 7] = [None, None, None, None, None, None, None];
        for l in header.lines() {
            let mut cleaned = l.trim();
            if cleaned.starts_with("//") {
                continue;
            } // its a commented line
            if let Some(com) = cleaned.find("//") {
                // remove comment after line
                cleaned = &cleaned[0..com].trim();
            }
            let (command, _value) =
                cleaned.split_at(cleaned.find("/").map(|x| x + 1).unwrap_or(cleaned.len()));
            match command {
                "$$BINARY" => items[0] = Some("0"),
                "$$ASCII" => items[0] = Some("1"),
                "$$UNITS/" => items[1] = Some(&cleaned["$$UNITS/".len()..]),
                "$$VERSION/" => items[2] = Some(&cleaned["$$VERSION/".len()..]),
                "$$LAYERS/" => items[5] = Some(&cleaned["$$LAYERS/".len()..]),
                "$$ALIGN" => items[6] = Some(""),
                _ => {}
            }
        }

        // Validate that all required header elements are present
        for req in 0u8..=2 {
            if items[req as usize].is_none() {
                Err(Error::HeaderIncomplete(req))?;
            }
        }

        Ok((
            search_index,
            Header {
                binary: items[0].unwrap() == "0", // We just checked not none
                units: items[1]
                    .unwrap()
                    .parse()
                    .map_err(|_| Error::InvalidHeaderValue)?,
                version: items[2]
                    .unwrap()
                    .parse::<f32>()
                    .map(|x| x / 100.0)
                    .map_err(|_| Error::InvalidHeaderValue)?,
                aligned: items[6].is_some(),
                layers: if let Some(l) = items[5] {
                    Some(l.parse::<usize>().map_err(|_| Error::InvalidHeaderValue)?)
                } else {
                    None
                },
            },
        ))
    }

    fn next_element(
        &mut self,
        current_layer: &mut Option<usize>,
        buf: &mut &'a [u8],
    ) -> Result<bool, Error> {
        // TODO: Should be some way to do this at compile time
        let aligned = self.header.aligned;
        let coord_size: usize = size_of::<<T as CLIType>::Coord>();
        let meta_size: usize = size_of::<<T as CLIType>::Meta>();
        // Implementation notes:
        // the CLI spec does not actually make clear what should happen to the last element in a 32bit aligned
        // data section. You could technically leave the last two empty bytes off of the end of the file and still have valid data.
        // Its also unlikely that this would ever matter since the last element of a data section is likely to be a hatches or polyline
        // command which would not end with a half word element. The only reason to leave the aggressive EOF check in is that without it,
        // get_meta could panic when advancing

        let cmd = buf.get_u16_le();
        if aligned {
            buf.advance(2)
        };

        match cmd {
            // Start layer long
            127 | 128 => {
                if cmd != T::CMD_LAYER {
                    Err(Error::TypeMismatch)?;
                }

                CLI::<T>::expect_eof(buf, coord_size + aligned as usize * 2)?;
                let l = Layer {
                    height: <T as CLIType>::get_coord(buf, aligned),
                    loops: vec![],
                    hatches: vec![],
                };
                // println!("New layer at: {:?}mm", l.height);
                self.layers.push(l);
                if let Some(layer) = current_layer {
                    *current_layer = Some(*layer + 1);
                } else {
                    *current_layer = Some(0);
                }
            }
            129 | 130 => {
                if cmd != T::CMD_PLINE {
                    Err(Error::TypeMismatch)?;
                }

                CLI::<T>::expect_eof(buf, 3 * (meta_size + aligned as usize * 2))?;
                let id = T::get_meta(buf, aligned);
                let dir = T::get_meta(buf, aligned);
                let n_pts = T::get_usize(buf, aligned) * 2; // num_pts * floats in point

                // $$ ALIGN not a factor here since the spec says should be tightly packed
                CLI::<T>::expect_eof(buf, coord_size * n_pts)?;
                let points = CLI::<T>::cast_slice(n_pts, buf);
                buf.advance(coord_size * n_pts);

                if let Some(l) = current_layer {
                    self.layers[*l].loops.push(Loop { id, dir, points });
                } else {
                    Err(Error::ElementOutsideLayer)?;
                }
            }
            // hatches short
            131 | 132 => {
                if cmd != T::CMD_HATCH {
                    Err(Error::TypeMismatch)?;
                }

                CLI::<T>::expect_eof(buf, 2 * (meta_size + aligned as usize * 2))?;
                let id = T::get_meta(buf, aligned);
                let n_pts = T::get_usize(buf, aligned) * 4; // num_pts * floats in point

                // $$ ALIGN not a factor here since the spec says should be tightly packed
                CLI::<T>::expect_eof(buf, coord_size * n_pts)?;
                let points = CLI::<T>::cast_slice(n_pts, buf);
                buf.advance(coord_size * n_pts);

                if let Some(l) = current_layer {
                    self.layers[*l].hatches.push(Hatches { id, points });
                } else {
                    Err(Error::ElementOutsideLayer)?;
                }
            }
            _ => return Err(Error::InvalidGeometryCommand(cmd)),
        }
        return Ok(buf.len() > 0);
    }

    fn cast_slice<A>(count: usize, floats: &'a [u8]) -> &'a [A] {
        unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const _, count) }
    }

    fn expect_eof(buf: &[u8], req_bytes: usize) -> Result<(), Error> {
        if buf.len() < req_bytes {
            Err(Error::UnexpectedEOF)
        } else {
            Ok(())
        }
    }

    /// Iterate over each layer in the file
    pub fn iter(&'a self) -> std::slice::Iter<'a, Layer<'a, T>> {
        self.layers.iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn header() -> Result<(), Error> {
        let data = r#"
$$HEADERSTART
// This is a example for the use of the Layer Format //
$$ASCII     
$$VERSION/105 
$$UNITS/1              // all coordinates are given in mm  // 
// $$UNITS/0.01     all coordinates are given in units 0.01 mm //      
$$DATE/070493                       // 7. April 1993 //
$$LAYERS/100                        //  100 layers //
$$HEADEREND                               

$$GEOMETRYSTART          // start of GEOMETRY-section//
"#;

        let (_, header) = CLI::<LongCLI>::parse_header(data.as_bytes())?;
        assert_eq!(header.units, 1.0);
        assert_eq!(header.version, 1.05);
        Ok(())
    }
}
