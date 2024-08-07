// Copyright 2024 Chris Ridd <chrisridd@mac.com>. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, Write};
use std::path::PathBuf;

use clap;
use clap::{command, crate_authors, crate_version, value_parser, Arg, Command};
use yazi::*;

/// Application error handling
enum ThemeError {
    IO(io::Error),
    Format(String),
    Zlib(yazi::Error),
}

impl Display for ThemeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::IO(e) => writeln!(f, "I/O error: {}", e),
            ThemeError::Format(s) => writeln!(f, "Bad format: {}", s),
            ThemeError::Zlib(e) => writeln!(f, "Decompress: {:?}", e),
        }
    }
}

impl From<io::Error> for ThemeError {
    fn from(error: io::Error) -> Self {
        ThemeError::IO(error)
    }
}

struct ResourceHeader {
    size: u32,            // offset 0
    offset: u32,          // offset 4
    compressed_size: u32, // offset 8
    name: String,         // offset 12
                          // extra NUL bytes padding to the nearest 4 bytes
}

enum ResourceKind {
    Config(String),
    Bitmap(String),
    TrueType(String),
    OpenType(String),
    Json(String),
    Unknown(String),
}

impl ResourceKind {
    pub fn kind_of(header: &ResourceHeader, bytes: &[u8]) -> ResourceKind {
        if header.name.is_empty() {
            return ResourceKind::Config(String::from("Configuration"));
        }
        if bytes.len() > 8 {
            let w = u16::from_le_bytes([bytes[0], bytes[1]]);
            let h = u16::from_le_bytes([bytes[2], bytes[3]]);
            let rawbpp = u16::from_le_bytes([bytes[4], bytes[5]]);
            let bpp = rawbpp & 0x7fff;
            let transparent = if rawbpp >= 0x8000 {
                " *"
            } else {
                ""
            };
            if w > 0 && w < 4096 && h > 0 && h < 4096 && bpp > 0 && bpp < 256 {
                return ResourceKind::Bitmap(format!("Bitmap {} x {} {}bpp{}", w, h, bpp, transparent));
            }
        }
        if bytes.len() > 2 {
            if bytes.starts_with("{\"".as_bytes()) {
                return ResourceKind::Json(String::from("JSON?"));
            }
        }
        if bytes.len() > 4 {
            if bytes.starts_with("OTTO".as_bytes()) {
                return ResourceKind::OpenType(String::from("OpenType font?"));
            }
        }
        if bytes.len() > 4 {
            if bytes.starts_with(&[0x00, 0x01, 0x00, 0x00]) {
                return ResourceKind::TrueType(String::from("TrueType font?"));
            }
        }
        ResourceKind::Unknown(String::from("Unknown"))
    }

    pub fn to_string(&self) -> &String {
        match self {
            ResourceKind::Config(str) => &str,
            ResourceKind::Bitmap(str) => &str,
            ResourceKind::TrueType(str) => &str,
            ResourceKind::OpenType(str) => &str,
            ResourceKind::Json(str) => &str,
            ResourceKind::Unknown(str) => &str,
        }
    }
}

fn read_headers<R: Read + Seek>(reader: &mut R) -> Result<Vec<ResourceHeader>, ThemeError> {
    let mut fingerprint = [0u8; 15];
    reader.read_exact(&mut fingerprint)?;
    if fingerprint != "PocketBookTheme".as_bytes() {
        return Err(ThemeError::Format(String::from(
            "File does not start PocketBookTheme",
        )));
    }
    let mut version = [0u8; 1];
    reader.read_exact(&mut version)?;
    if version[0] != 1 {
        return Err(ThemeError::Format(String::from("Not version 1")));
    }
    let mut header_len = [0u8; 4];
    reader.read_exact(&mut header_len)?;
    let endpos = reader.stream_position().unwrap() + u32::from_le_bytes(header_len) as u64 - 32u64;
    let mut headers = vec![];
    while reader.stream_position().unwrap() < endpos {
        let header = read_resource_header(reader, headers.is_empty())?;
        headers.push(header);
    }

    Ok(headers)
}

fn read_resource<R: Read + Seek>(
    reader: &mut R,
    header: &ResourceHeader,
) -> Result<Box<[u8]>, ThemeError> {
    reader.seek(io::SeekFrom::Start(header.offset as u64))?;
    let mut compressed = vec![0u8; header.compressed_size as usize];
    reader.read_exact(&mut compressed)?;
    match decompress(&compressed, Format::Zlib) {
        Ok((uncompressed, _)) => Ok(uncompressed.into_boxed_slice()),
        Err(e) => Err(ThemeError::Zlib(e)),
    }
}

fn read_resource_header<R: Read>(
    reader: &mut R,
    first: bool,
) -> Result<ResourceHeader, ThemeError> {
    let mut size = [0u8; 4];
    let mut unknown = [0u8; 4];
    let mut compressed_size = [0u8; 4];
    reader.read_exact(&mut size)?;
    reader.read_exact(&mut unknown)?;
    reader.read_exact(&mut compressed_size)?;
    let mut name = String::from("");
    while !first {
        let mut chars = [0u8; 4];
        reader.read_exact(&mut chars)?;
        if chars[0] == 0 {
            break;
        }
        name.push(chars[0] as char);
        if chars[1] == 0 {
            break;
        }
        name.push(chars[1] as char);
        if chars[2] == 0 {
            break;
        }
        name.push(chars[2] as char);
        if chars[3] == 0 {
            break;
        }
        name.push(chars[3] as char);
    }
    Ok(ResourceHeader {
        size: u32::from_le_bytes(size),
        offset: u32::from_le_bytes(unknown),
        compressed_size: u32::from_le_bytes(compressed_size),
        name: name,
    })
}

fn list(themefile: &PathBuf) -> () {
    let mut reader: BufReader<File> = match File::open(themefile) {
        Err(why) => {
            eprintln!("couldn't open {}: {}", themefile.display(), why);
            return;
        }
        Ok(file) => BufReader::new(file),
    };

    match read_headers(&mut reader) {
        Err(why) => {
            eprintln!("Error: {}", why);
            return;
        }
        Ok(headers) => {
            // loop through first to find column widths
            let mut verboses = Vec::new();
            let mut max_resource = "resource".len();
            let mut max_size = 9999usize;
            let mut max_csize = 999999999999999usize;
            let mut max_verbose = "verbose".len();
            for header in &headers {
                let res = match read_resource(&mut reader, header) {
                    Ok(res) => res,
                    Err(e) => panic!("Error {}", e),
                };
                let kind = ResourceKind::kind_of(header, &res);
                verboses.push(kind.to_string().clone());
                if header.name.len() > max_resource {
                    max_resource = header.name.len();
                }
                if header.size as usize > max_size {
                    max_size = header.size as usize;
                }
                if header.compressed_size as usize > max_csize {
                    max_csize = header.compressed_size as usize;
                }
                if kind.to_string().len() > max_verbose {
                    max_verbose = kind.to_string().len();
                }
            }
            // Loop through second time to print the table
            max_size = digits_len(max_size);
            max_csize = digits_len(max_csize);
            println!("{:<rwidth$}  {:>swidth$}  {:>cwidth$}  {}",
                     "resource",
                     "size",
                     "compressed size",
                     "verbose",
                     rwidth = max_resource, swidth = max_size, cwidth = max_csize,
            );
            println!("{}",
                     "-".repeat(max_resource + 2 + max_size + 2 + max_csize + 2 + max_verbose));
            let mut i = 0;
            for header in &headers {
                println!("{:<rwidth$}  {:>swidth$}  {:>cwidth$}  {}",
                         header.name, header.size, header.compressed_size, verboses[i],
                         rwidth = max_resource, swidth = max_size, cwidth = max_csize,
                );
                i = i + 1;
            }
        }
    }
}

fn digits_len(val: usize) -> usize {
    let str = format!("{}", val);
    str.len()
}

fn unpack(themefile: &PathBuf, resources: Vec<&String>) -> () {
    let mut reader: BufReader<File> = match File::open(themefile) {
        Err(why) => {
            eprintln!("couldn't open {}: {}", themefile.display(), why);
            return;
        }
        Ok(file) => BufReader::new(file),
    };

    match read_headers(&mut reader) {
        Err(why) => {
            eprintln!("Error: {}", why);
            return;
        }
        Ok(headers) => {
            for header in &headers {
                if resources.contains(&&header.name) {
                    let res = match read_resource(&mut reader, header) {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("Error {}", e);
                            return;
                        }
                    };
                    let filename = if header.name.is_empty() {
                        PathBuf::from("theme.cfg")
                    } else {
                        PathBuf::from(&header.name)
                    };
                    let mut file = match File::create(filename) {
                        Err(e) => {
                            eprintln!("Error creating file {}", e);
                            return;
                        }
                        Ok(file) => BufWriter::new(file),
                    };
                    match file.write_all(&*res) {
                        Err(e) => {
                            eprintln!("Error writing file {}", e);
                            return;
                        }
                        Ok(_) => { },
                    }
                }
            }
        }
    }
}

fn main() {
    let args = command!()
        .about("List PocketBook themes and extract theme resources")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .subcommand(
            Command::new("-l").about("List theme resources").arg(
                Arg::new("theme-file")
                    .value_parser(value_parser!(PathBuf))
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("-u")
                .about("Unpack theme resources")
                .arg(
                    Arg::new("theme-file")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(Arg::new("resource-name").required(true).num_args(1..)),
        )
        .disable_help_subcommand(true)
        .get_matches();

    if let Some(list_args) = args.subcommand_matches("-l") {
        list(list_args.get_one::<PathBuf>("theme-file").unwrap());
    } else if let Some(unpack_args) = args.subcommand_matches("-u") {
        unpack(
            unpack_args.get_one::<PathBuf>("theme-file").unwrap(),
            unpack_args.get_many::<String>("resource-name")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        );
    }
}
