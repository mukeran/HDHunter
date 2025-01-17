use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use libafl_bolts::{ownedref::OwnedSlice, AsSlice};

use super::HttpInput;

const MAX_CHUNK_SIZE: usize = 0x2000;

fn to_cgi_style_response(input: &HttpInput) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let status_line = unsafe { input.first_line().child_unchecked(0) };
    result.extend_from_slice(b"Status:");
    for i in 1..6 {
        result.extend_from_slice(unsafe { status_line.child_unchecked(i) }.bytes().as_slice());
    }
    for i in 1..4 {
        result.extend_from_slice(unsafe { input.node().child_unchecked(i) }.bytes().as_slice());
    }
    result
}

pub fn to_scgi(input: &HttpInput) -> OwnedSlice<u8> {
    OwnedSlice::from(to_cgi_style_response(input))
}

pub fn to_fastcgi(input: &HttpInput) -> OwnedSlice<u8> {
    let cgi_response = to_cgi_style_response(input);

    let mut result: Vec<u8> = Vec::new();

    // FCGI_STDOUT: chunks
    for chunk in cgi_response.chunks(MAX_CHUNK_SIZE) {
        result.write_u8(1).unwrap(); result.write_u8(6).unwrap();
        result.write_u16::<BigEndian>(1).unwrap(); result.write_u16::<BigEndian>(chunk.len() as u16).unwrap();
        result.write_u8(0).unwrap(); result.write_u8(0).unwrap();
        result.extend_from_slice(chunk);
    }

    // FCGI_STDOUT: last chunk
    result.write_u8(1).unwrap(); result.write_u8(6).unwrap();
    result.write_u16::<BigEndian>(1).unwrap(); result.write_u16::<BigEndian>(0).unwrap();
    result.write_u8(0).unwrap(); result.write_u8(0).unwrap();

    // FCGI_END_REQUEST
    result.write_u8(1).unwrap(); result.write_u8(3).unwrap();
    result.write_u16::<BigEndian>(1).unwrap(); result.write_u16::<BigEndian>(8).unwrap();
    result.write_u8(0).unwrap(); result.write_u8(0).unwrap();
    result.write_u32::<BigEndian>(0).unwrap(); result.write_u8(0).unwrap(); result.write_all(&[0, 0, 0]).unwrap();

    OwnedSlice::from(result)
}

pub fn to_uwsgi(input: &HttpInput) -> OwnedSlice<u8> {
    input.node().bytes()
}

fn ajp_encode_string(data: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.write_u16::<BigEndian>(data.len() as u16).unwrap();
    result.extend_from_slice(data);
    result.write_u8(0).unwrap();
    result
}

fn ajp_make_packet(data: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.write_u8(0x41).unwrap();
    result.write_u8(0x42).unwrap();
    result.write_u16::<BigEndian>(data.len() as u16).unwrap();
    result.extend_from_slice(data);
    result
}

pub fn to_ajp(input: &HttpInput) -> OwnedSlice<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();

    // AJP13_SEND_HEADERS
    let status_line = unsafe { input.first_line().child_unchecked(0) };
    let status_code = std::str::from_utf8(unsafe { status_line.child_unchecked(2) }.bytes().as_slice()).unwrap_or("200").parse::<u16>().unwrap_or(200);
    let status_msg = unsafe { status_line.child_unchecked(2) }.bytes();
    buffer.write_u8(4).unwrap();
    buffer.write_u16::<BigEndian>(status_code).unwrap();
    buffer.extend_from_slice(&ajp_encode_string(status_msg.as_slice()));
    
    let field_lines = input.field_lines().children();
    buffer.write_u16::<BigEndian>(field_lines.len() as u16).unwrap();
    for field_line in field_lines {
        let field_name = unsafe { field_line.child_unchecked(0) }.bytes();
        let field_value = unsafe { field_line.child_unchecked(3) }.bytes();
        buffer.extend_from_slice(&ajp_encode_string(field_name.as_slice()));
        buffer.extend_from_slice(&ajp_encode_string(field_value.as_slice()));
    }
    result.extend_from_slice(&ajp_make_packet(buffer.as_slice()));
    buffer.clear();

    // AJP13_SEND_BODY_CHUNK
    let message_body = input.message_body().bytes();
    for chunk in message_body.chunks(MAX_CHUNK_SIZE) {
        buffer.write_u8(3).unwrap();
        buffer.write_u16::<BigEndian>(chunk.len() as u16).unwrap();
        buffer.extend_from_slice(chunk);
        buffer.write_u8(0).unwrap();

        result.extend_from_slice(&ajp_make_packet(buffer.as_slice()));
        buffer.clear();
    }

    // AJP13_END_RESPONSE
    buffer.write_u8(5).unwrap();
    buffer.write_u8(0).unwrap();
    result.extend_from_slice(&ajp_make_packet(buffer.as_slice()));

    OwnedSlice::from(result)
}

#[cfg(test)]
mod tests {
    use libafl_bolts::AsSlice;

    use crate::{input::{HttpInput, Node, NodeType}, new_node};

    fn input() -> HttpInput {
        let start_line = new_node!(
            NodeType::StartLine,
            new_node!(
                NodeType::StatusLine,
                new_node!(NodeType::RawBytes; b"HTTP/1.1".into()),
                new_node!(NodeType::SP; b" ".into()),
                new_node!(NodeType::RawBytes; b"200".into()),
                new_node!(NodeType::SP; b" ".into()),
                new_node!(NodeType::RawBytes; b"OK".into()),
                new_node!(NodeType::CRLF; b"\r\n".into()),
            ),
        );
        let field_lines = new_node!(
            NodeType::FieldLines,
            new_node!(
                NodeType::FieldLine,
                new_node!(NodeType::RawBytes; b"Content-Type".into()),
                new_node!(NodeType::COLON; b":".into()),
                new_node!(NodeType::OWSBWS; b" ".into()),
                new_node!(NodeType::RawBytes; b"text/html".into()),
                new_node!(NodeType::OWSBWS; b"".into()),
                new_node!(NodeType::CRLF; b"\r\n".into()),
            ),
        );
        let message_body = new_node!(
            NodeType::MessageBody,
            new_node!(NodeType::RawBytes; b"<html><body>Hello, world!</body></html>".into()),
        );
        let input = HttpInput::new(new_node!(
            NodeType::HttpMessage,
            start_line,
            field_lines,
            new_node!(NodeType::CRLF; b"\r\n".into()),
            message_body,
        )).unwrap();
        input
    }

    #[test]
    fn test_to_scgi() {
        let input = input();
        let result = super::to_scgi(&input);
        println!("{:?}", unsafe { std::str::from_utf8_unchecked(result.as_slice()) });
    }

    #[test]
    fn test_to_fastcgi() {
        let input = input();
        let result = super::to_fastcgi(&input);
        println!("{:?}", unsafe { std::str::from_utf8_unchecked(result.as_slice()) });
    }

    #[test]
    fn test_to_uwsgi() {
        let input = input();
        let result = super::to_uwsgi(&input);
        println!("{:?}", unsafe { std::str::from_utf8_unchecked(result.as_slice()) });
    }

    #[test]
    fn test_to_ajp() {
        let input = input();
        let result = super::to_ajp(&input);
        println!("{:?}", unsafe { std::str::from_utf8_unchecked(result.as_slice()) });
    }
}