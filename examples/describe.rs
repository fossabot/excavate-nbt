//extern crate excavate_nbt;
extern crate nbt;

use nbt::Tag;
use nbt::read_file;

use std::fmt;
use std::fmt::Write;
use std::fmt::Display;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;

// TODO: Rewrite at a later date to be much cleaner
#[derive(Clone,PartialEq)]
struct Describe {
    tag: Tag,
    name: Option<String>,
    indent_level: usize,
}

fn main() {
    match std::env::args().nth(1) {
        Some(ref source) if source == "--" => {
            let mut reader = BufReader::new(std::io::stdin());
            let file = read_file(&mut reader).unwrap();
            println!("Source: stdin");
            println!("Compression: {}", file.compression.to_str());
            println!("{}", Describe {
                tag: file.root,
                name: None,
                indent_level: 0,
            });
        },
        Some(ref path) => {
            let path = Path::new(&path);
            let mut reader = File::open(path).map(BufReader::new).expect("Failed to open NBT file");
            let file = read_file(&mut reader).unwrap();
            println!("Source: {}", path.to_str().unwrap());
            println!("Compression: {}", file.compression.to_str());
            println!("{}", Describe {
                tag: file.root,
                name: None,
                indent_level: 0,
            });
        },
        None => {
            println!("Usage: cat file.nbt | describe --");
            println!("Usage: describe /path/to/file.nbt");
            std::process::exit(0);
        }
    }
}

impl Describe {
    fn contains_tags(&self) -> bool {
        match &self.tag {
            &Tag::Compound(..) |
            &Tag::List(..) => true,
            _ => false
        }
    }

    fn indent(indent_level: usize, f: &mut fmt::Formatter) -> fmt::Result {
        if indent_level < 1 {
            return Ok(())
        }

        let mut padding = String::with_capacity(indent_level * 2);
        for _ in 0..indent_level {
            padding.push_str("  ");
        }
        f.pad(&padding)
    }
}

impl Display for Describe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.tag {
            Tag::Byte(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::Short(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::Int(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::Long(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::Float(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::Double(x) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("{}", x))
            },
            Tag::ByteArray(ref contents) => {
                if None == self.name {
                    f.write_fmt(format_args!("ByteArray : {} bytes", contents.len()))?;
                }

                let mut counter = 0;
                let mut content_iter = contents.split(|_| {
                    counter += 1;
                    counter % 16 == 0
                }).peekable();

                while let Some(chunk) = content_iter.next() {
                    f.write_char('\n')?;
                    Describe::indent(self.indent_level + 1, f)?;
                    let mut chunk_iter = chunk.iter().peekable();
                    while let Some(b) = chunk_iter.next() {
                        f.write_fmt(format_args!("{:02X}", b))?;
                        if let Some(..) = chunk_iter.peek() {
                            f.write_str(", ")?;
                        }
                    }
                }
                Ok(())
            }
            Tag::String(text) => {
                if None == self.name {
                    f.write_fmt(format_args!("{} ", self.tag.type_name()))?;
                }
                f.write_fmt(format_args!("'{}'", text))
            },
            Tag::List(ref tag_list) => {
                if None == self.name {
                    f.write_fmt(format_args!("List : {} entry\n", tag_list.len()))?;
                }

                tag_list.iter().map(|tag| {
                    Describe {
                        tag: tag.clone(),
                        name: None,
                        indent_level: self.clone().indent_level + 1
                    }
                }).try_for_each(|describe| {
                    Describe::indent(describe.indent_level, f)?;
                    f.write_fmt(format_args!("{}", describe))?;
                    match describe.tag {
                        Tag::Compound(..) |
                        Tag::List(..)  => Ok(()),
                        _ => f.write_char('\n')
                    }
                })
            }
            Tag::Compound(ref tag_mappings) => {
                if None == self.name {
                    f.write_fmt(format_args!("Compound : {} entry\n", tag_mappings.len()))?;
                }

                tag_mappings.iter().filter(|(_, t)| t != &Tag::End).map(|(ref name, ref tag)| {
                    Describe {
                        tag: tag.clone(),
                        indent_level: self.indent_level + 1,
                        name: match name.len() {
                            x if x > 0 => Some(name.clone()),
                            _ => None
                        }
                    }
                }).try_for_each(|describe| {
                    Describe::indent(describe.indent_level, f)?;
                    if let Some(ref name) = describe.name {
                        f.write_fmt(format_args!("Named {} '{}' ", describe.tag.type_name(), name))?;
                    } else {
                        f.write_fmt(format_args!("{}", describe.tag.type_name()))?;
                    }

                    if describe.contains_tags() {
                        f.write_char('\n')?;
                    }

                    f.write_fmt(format_args!("{}", describe))?;

                    if !describe.contains_tags() {
                        f.write_char('\n')?;
                    }
                    Ok(())
                })
            },
            Tag::IntArray(ref contents) => {
                if None == self.name {
                    f.write_fmt(format_args!("IntArray : {} entry\n", contents.len()))?;
                }

                for i in contents {
                    Describe::indent(self.indent_level + 1, f)?;
                    f.write_fmt(format_args!("{}\n", i))?;
                }
                Ok(())
            },
            Tag::LongArray(ref contents) => {
                if None == self.name {
                    f.write_fmt(format_args!("LongArray : {} entry\n", contents.len()))?;
                }

                for i in contents {
                    Describe::indent(self.indent_level + 1, f)?;
                    f.write_fmt(format_args!("{}\n", i))?;
                }
                Ok(())
            },
            _ => Ok(())
        }
    }
}