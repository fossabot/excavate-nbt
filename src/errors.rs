
error_chain! {
    errors {
        InvalidTagType(id: u8) {
            description("Tag is not part of the NBT specification")
            display("invalid tag type: '0x{:02X}'", id)
        }
        InvalidCompressionFormat(format_header: u8) {
            description("invalid compression format"),
            display("invalid compression format: '0x{:02X}'", format_header)
        }
        UnexpectedElement(container_type: String, element_type: u8) {
            description("tags are heterogeneous")
            display("invalid element in {}: '0x{:02X}'", container_type, element_type)
        }
        InvalidHeaderLength(expected: i32) {
            description("invalid header length"),
            display("header length mismatch, expected length of {}", expected)
        }
    }

    foreign_links {
        Io(::std::io::Error);
        FromUtf8(::std::string::FromUtf8Error);
    }
}