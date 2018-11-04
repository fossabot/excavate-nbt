use std::io;
use std::io::Read;
use std::io::BufRead;
use byteorder::ReadBytesExt;
use errors::*;

use tag::Tag;
use tag::Compression;
use tag::DesktopEndianness;

#[derive(Debug)]
/// Structure representing a NBT file that has been fully parsed
pub struct NBTFile {
    pub compression: Compression,
    pub root: Tag
}

pub fn read_file<R: BufRead>(reader: &mut R) -> Result<NBTFile> {
    let compression_header = if let Ok(buf) = reader.fill_buf() {
        buf[0]
    } else {
        bail!("Failed to peek Compression type")
    };

    let compression = Compression::from_byte(compression_header).unwrap();
    let root = match compression {
        Compression::Zlib => read_compound(&mut ::flate2::read::ZlibDecoder::new(reader)),
        Compression::Gzip => read_compound(&mut ::flate2::read::GzDecoder::new(reader)),
        Compression::None => read_compound(&mut ::flate2::read::GzDecoder::new(reader)),
    }?;

    Ok(NBTFile {
        compression,
        root
    })
}

pub fn read_compound<R: Read>(reader: &mut R) -> Result<Tag> {
    let mut container = Vec::new();

    loop {
        let tag_type = {
            let mut buf: [u8; 1] = [0];
            if let Err(e) = reader.read_exact(&mut buf) {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    break;
                }
                bail!(e)
            }
            buf[0]
        };

        if tag_type == 0x00 {
            container.push((String::with_capacity(0), Tag::End));
            break;
        }

        let tag_name = match read_string(reader)? {
            Tag::String(tag_name) => tag_name,
            _ => bail!("failed to read compound tag name")
        };

        let tag_value = match tag_type {
            0x01 => read_byte(reader),
            0x02 => read_short(reader),
            0x03 => read_int(reader),
            0x04 => read_long(reader),
            0x05 => read_float(reader),
            0x06 => read_double(reader),
            0x07 => read_byte_array(reader),
            0x08 => read_string(reader),
            0x09 => read_list(reader),
            0x0A => read_compound(reader),
            0x0B => read_int_array(reader),
            0x0C => read_long_array(reader),
            data_type => Err(ErrorKind::InvalidTagType(data_type).into())
        }?;

        container.push((tag_name, tag_value));
    }
    Ok(Tag::Compound(container))
}

#[inline]
pub fn read_byte<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Byte(reader.read_i8()?))
}

#[inline]
pub fn read_short<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Short(reader.read_i16::<DesktopEndianness>()?))
}

#[inline]
pub fn read_int<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Int(reader.read_i32::<DesktopEndianness>()?))
}

#[inline]
pub fn read_long<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Long(reader.read_i64::<DesktopEndianness>()?))
}

#[inline]
pub fn read_float<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Float(reader.read_f32::<DesktopEndianness>()?))
}

#[inline]
pub fn read_double<R: Read>(reader: &mut R) -> Result<Tag> {
    Ok(Tag::Double(reader.read_f64::<DesktopEndianness>()?))
}

pub fn read_byte_array<R: Read>(reader: &mut R) -> Result<Tag> {
    let length = match read_int(reader)? {
        Tag::Int(len) if len < 0 => Err(ErrorKind::InvalidHeaderLength(len)),
        Tag::Int(len) => Ok(len),
        _ => bail!("failed to read array length"),
    }?;

    let mut buf: Vec<i8> = Vec::with_capacity(length as usize);
    for i in 0..length {
        match read_byte(reader) {
            Ok(Tag::Byte(val)) => {
                buf.push(val);
                Ok(())
            },
            Ok(tag) => Err(ErrorKind::UnexpectedElement(String::from("ByteArray"), tag.type_id())),
            Err(..) if i < length => Err(ErrorKind::InvalidHeaderLength(i)),
            Err(e) => Err(e.into()),
        }?;
    }
    Ok(Tag::ByteArray(buf))
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<Tag> {
    /*
     * https://wiki.vg/NBT#Specification
     *
     * NBT strings are prefixed with *unsigned* 16-bit numbers
     * despite the rest of the format using signed numbers
     */
    let length = reader.read_u16::<DesktopEndianness>()? as usize;
    let mut buf = Vec::with_capacity(length);
    let read = reader.take(length as u64).read_to_end(&mut buf)?;
    if read != length {
        bail!("Failed to read expected number of bytes")
    }
    let value = String::from_utf8(buf)?;
    Ok(Tag::String(value))
}

pub fn read_list<R: Read>(reader: &mut R) -> Result<Tag> {
    let mut tag_type: [u8; 1] = [0];
    reader.read_exact(&mut tag_type)?;

    if let Tag::Int(length) = read_int(reader)? {
        let mut buf = Vec::with_capacity(length as usize);
        for _ in 0..length {
            buf.push(match tag_type[0] {
                0x00 => Ok(Tag::End),
                0x01 => read_byte(reader),
                0x02 => read_short(reader),
                0x03 => read_int(reader),
                0x04 => read_long(reader),
                0x05 => read_float(reader),
                0x06 => read_double(reader),
                0x07 => read_byte_array(reader),
                0x08 => read_string(reader),
                0x09 => read_list(reader),
                0x0A => read_compound(reader),
                0x0B => read_int_array(reader),
                0x0C => read_long_array(reader),
                data_type => Err(ErrorKind::UnexpectedElement(String::from("List"), data_type).into()),
            }?);
        }
        return Ok(Tag::List(buf))
    }
    Err("Failed to read List Length header".into())
}

pub fn read_int_array<R: Read>(reader: &mut R) -> Result<Tag> {
    if let Tag::Int(length) = read_int(reader)? {
        let mut array_contents = Vec::with_capacity(length as usize);
        for _ in 0..length {
            match read_int(reader) {
                Ok(Tag::Int(value)) => array_contents.push(value),
                _ => bail!("Failed to read array contents")
            }
        }
        return Ok(Tag::IntArray(array_contents))
    }
    bail!("Failed to read array length")
}

pub fn read_long_array<R: Read>(reader: &mut R) -> Result<Tag> {
    if let Ok(Tag::Int(length)) = read_int(reader) {
        let mut array_contents = Vec::with_capacity(length as usize);
        for _ in 0..length {
            match read_long(reader) {
                Ok(Tag::Long(value)) => array_contents.push(value),
                _ => bail!("Failed to read array contents")
            }
        }
        return Ok(Tag::LongArray(array_contents))
    }
    bail!("Failed to read array length")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_reader() {
        let file = File::open(Path::new("./tests/hello_world.nbt"))
            .chain_err(|| "Unable to open hello_world.nbt");
        let mut reader = BufReader::new(file.unwrap());
        match read_compound(&mut reader) {
            Ok(Tag::Compound(mut tags)) => {
                let (root_element, root_tag) = tags.pop()
                    .expect("Expected TAG_End");

                assert_eq!(root_element, String::from("hello world"));

                let (name_name, name_tag) = match root_tag {
                    Tag::Compound(mut tags) => {
                        let end = tags.pop().map(|(_,b)| b).expect("TAG_End");
                        assert_eq!(Tag::End.type_id(), end.type_id());
                        tags.pop().expect("TAG_String")
                    },
                    _ => panic!("Child tag is not a Tag::Compound"),
                };

                if let Tag::String(value) = name_tag {
                    assert_eq!(name_name, "name");
                    assert_eq!(value, "Bananrama")
                } else {
                    panic!("Failed to read \"name\" tag")
                }
            },
            Ok(tag) => panic!("Expected Tag::Compound, got Tag::{}", tag.type_name()),
            _ => panic!("Expected a Tag::Compound"),
        }
    }

    #[test]
    fn test_bigtest() {
        // TODO: Improve test after improving fluency of codebase
        let file = File::open(Path::new("./tests/bigtest.nbt"))
            .chain_err(|| "Unable to open bigtest.nbt");
        let mut reader = BufReader::new(file.unwrap());
        read_file(&mut reader)
            .expect("Failed to parse complex NBT structure");
    }
}