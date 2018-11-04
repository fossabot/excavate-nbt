pub use byteorder::LittleEndian as PocketEndianness;

pub use byteorder::BigEndian as DesktopEndianness;
use std::str::FromStr;
use errors;

#[derive(Debug)]
/// The NBT specification accepts three different compression formats:
/// - none
/// - gzip
/// - zlib
pub enum Compression {
    None,
    Gzip ,
    Zlib ,
}

impl Compression {
    /// Get a human-readable name of the compression algorithm used
    pub fn to_str(&self) -> &str {
        match *self {
            Compression::None => "None",
            Compression::Gzip => "Gzip",
            Compression::Zlib => "Zlib",
        }
    }

    /// Parse the compression header byte to determine if our file
    /// has been deflated at all
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0x0a => Some(Compression::None),
            0x1f => Some(Compression::Gzip),
            0x78 => Some(Compression::Zlib),
            _ => None
        }
    }
}

impl FromStr for Compression {
    type Err = errors::Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "Gzip" => Ok(Compression::Gzip),
            "Zlib" => Ok(Compression::Zlib),
            "None" => Ok(Compression::None),
            _ => bail!("Invalid Compression type"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Tag {
    /// Indicates the end of an NBT List or Compound structure
    End,
    /// A single signed byte
    Byte(i8),
    /// A single signed, 16 bit integer
    Short(i16),
    /// A single signed, 32 bit integer
    Int(i32),
    /// A signed signed, 64 bit integer
    Long(i64),
    /// A single, IEEE-754 single-precision floating point number
    Float(f32),
    /// A single, IEEE-754 double-precision floating point number
    Double(f64),
    /// An array of signed bytes prefixed by a signed 32 bit integer
    ByteArray(Vec<i8>),
    /// A UTF8 encoded string prefixed by an **unsigned** 16 bit integer
    String(String),
    /// A list of **nameless** tags which are expected to all be of the same type,
    /// the list is prefixed by a single byte indicating the type and followed
    /// by a signed 32 bit integer indicating the number of elements
    List(Vec<Tag>),
    /// A list of **named** tags which can contain any type of tag, each key-value
    /// pair in a compound tag is prefixed by a single byte indicating the type
    /// of data contained in the pair. A Compound tag is read until it encounters
    /// an `End` tag.
    Compound(Vec<(String, Tag)>),
    /// An array of signed 32 bit integers prefixed by a signed 32 bit integer
    IntArray(Vec<i32>),
    /// An array of signed 64 bit integers prefixed by a signed 64 bit integer
    LongArray(Vec<i64>)
}

impl Tag {
    /// Get a human-readable name for the type of data stored by the tag
    pub fn type_name(&self) -> &str {
        match *self {
            Tag::End => "End",
            Tag::Byte(..) => "Byte",
            Tag::Short(..) => "Short",
            Tag::Int(..) => "Int",
            Tag::Long(..) => "Long",
            Tag::Float(..) => "Float",
            Tag::Double(..) => "Double",
            Tag::ByteArray(..) => "ByteArray",
            Tag::String(..) => "String",
            Tag::List(..) => "List",
            Tag::Compound(..) => "Compound",
            Tag::IntArray(..) => "IntArray",
            Tag::LongArray(..) => "LongArray",
        }
    }

    /// Get the byte used to discriminate encoded tags
    pub fn type_id(&self) -> u8 {
        match *self {
            Tag::End => 0x00,
            Tag::Byte(..) => 0x01,
            Tag::Short(..) => 0x02,
            Tag::Int(..) => 0x03,
            Tag::Long(..) => 0x04,
            Tag::Float(..) => 0x05,
            Tag::Double(..) => 0x06,
            Tag::ByteArray(..) => 0x07,
            Tag::String(..) => 0x08,
            Tag::List(..) => 0x09,
            Tag::Compound(..) => 0x0a,
            Tag::IntArray(..) => 0x0b,
            Tag::LongArray(..) => 0x0c,
        }
    }
}

// region impl From for Tag

impl From<i8> for Tag {
    fn from(val: i8) -> Self {
        Tag::Byte(val)
    }
}

impl From<i16> for Tag {
    fn from(val: i16) -> Self {
        Tag::Short(val)
    }
}

impl From<i32> for Tag {
    fn from(val: i32) -> Self {
        Tag::Int(val)
    }
}

impl From<i64> for Tag {
    fn from(val: i64) -> Self {
        Tag::Long(val)
    }
}

impl From<f32> for Tag {
    fn from(val: f32) -> Self {
        Tag::Float(val)
    }
}

impl From<f64> for Tag {
    fn from(val: f64) -> Self {
        Tag::Double(val)
    }
}

// endregion impl From for Tag