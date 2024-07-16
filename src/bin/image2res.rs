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
use std::io;
use std::io::Write;
use std::path::PathBuf;
use clap::*;
use image::{GenericImageView, ImageError, Pixel};
use image::io::Reader as ImageReader;

enum ConvertError {
    IO(std::io::Error),
    Format(String),
    Image(ImageError),
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::IO(e) => writeln!(f, "I/O error: {}", e),
            ConvertError::Format(s) => writeln!(f, "Bad format: {}", s),
            ConvertError::Image(e) => writeln!(f, "Image save error: {}", e),
        }
    }
}

impl From<io::Error> for ConvertError {
    fn from(error: io::Error) -> Self {
        ConvertError::IO(error)
    }
}

impl From<ImageError> for ConvertError {
    fn from(error: ImageError) -> Self {
        ConvertError::Image(error)
    }
}

fn convert(src: &PathBuf) -> Result<(),ConvertError> {
    let img = ImageReader::open(src)?.decode()?;
    let mut dst = src.clone();
    dst.set_extension("");

    let mut file = File::create(dst)?;
    let (w, h) = img.dimensions();
    if w > 0x7fff || h > 0x7fff {
        return Err(ConvertError::Format(String::from("too big")));
    }
    let w16 = w as u16;
    let h16 = h as u16;
    file.write_all(&w16.to_le_bytes())?;
    file.write_all(&h16.to_le_bytes())?;
    file.write_all(&24u16.to_le_bytes())?;
    let scanlinew = w16 * 3;
    file.write_all(&scanlinew.to_le_bytes())?;
    let mut scanline = vec![0u8; scanlinew as usize];
    for y in 0..h {
        for x in 0..w {
            let rgb = img.get_pixel(x, y);
            scanline[x as usize * 3 + 0] = rgb.channels()[0];
            scanline[x as usize * 3 + 1] = rgb.channels()[1];
            scanline[x as usize * 3 + 2] = rgb.channels()[2];
        }
        file.write_all(&scanline)?;
    }
    Ok(())
}

fn main() {
    let args = command!()
        .about("Convert images into PocketBook theme image resources")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .arg(
            Arg::new("resource-file")
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .get_matches();

    match convert(args.get_one("resource-file").unwrap()) {
        Ok(_) => std::process::exit(0),
        Err(e) => eprintln!("Failed {}", e),
    }
}
