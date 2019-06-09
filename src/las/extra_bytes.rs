/*
===============================================================================

  PROGRAMMERS:

    martin.isenburg@rapidlasso.com  -  http://rapidlasso.com
    uday.karan@gmail.com - Hobu, Inc.
    andrew.bell.ia@gmail.com - Hobu Inc.

  COPYRIGHT:

    (c) 2007-2014, martin isenburg, rapidlasso - tools to catch reality
    (c) 2014, Uday Verma, Hobu, Inc.
    (c) 2019, Thomas Montaigu

    This is free software; you can redistribute and/or modify it under the
    terms of the GNU Lesser General Licence as published by the Free Software
    Foundation. See the COPYING file for more information.

    This software is distributed WITHOUT ANY WARRANTY and without even the
    implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

  CHANGE HISTORY:
    6 June 2019: Translated to Rust
===============================================================================
*/


use crate::models::{ArithmeticModel, ArithmeticModelBuilder};

use crate::formats::{FieldCompressor, FieldDecompressor};
use crate::encoders::ArithmeticEncoder;
use crate::decoders::ArithmeticDecoder;
use std::io::{Write, Read};

#[derive(Debug)]
pub struct ExtraBytes {
    bytes: Vec<u8>
}


pub struct ExtraBytesCompressor {
    have_last: bool,
    count: usize,
    lasts: Vec<u8>,
    diffs: Vec<u8>,
    models: Vec<ArithmeticModel>,
}


impl ExtraBytesCompressor {
    pub fn new(count: usize) -> Self {
        Self {
            have_last: false,
            count,
            lasts: vec![0u8; count],
            diffs: vec![0u8; count],
            models: (0..count).into_iter().map(|i| ArithmeticModelBuilder::new(256).build()).collect(),
        }
    }
}


impl<W: Write> FieldCompressor<W> for ExtraBytesCompressor {
    fn size_of_field(&self) -> usize {
        self.count
    }

    fn compress_with(&mut self, encoder: &mut ArithmeticEncoder<W>, buf: &[u8]) {
        for i in 0..self.count {
            let current_byte = &buf[i];
            let last = &mut self.lasts[i];

            self.diffs[i] = *current_byte - *last;
            *last = *current_byte;
        }

        if !self.have_last {
            encoder.out_stream().write_all(&self.lasts).unwrap();
            self.have_last = true;
        } else {
            for (diff, mut model) in self.diffs.iter().zip(self.models.iter_mut()) {
                encoder.encode_symbol(&mut model, *diff as u32);
            }
        }
    }
}

pub type ExtraBytesDecompressor = ExtraBytesCompressor;

impl<R: Read> FieldDecompressor<R> for ExtraBytesDecompressor {
    fn size_of_field(&self) -> usize {
        self.count
    }

    fn decompress_with(&mut self, decoder: &mut ArithmeticDecoder<R>, mut buf: &mut [u8]) {
        if !self.have_last {
            decoder.in_stream().read_exact(&mut buf).unwrap();
            self.lasts.copy_from_slice(buf);
            self.have_last = true;
        } else {
            for i in 0..self.count {
                let diff = &mut self.diffs[i];
                let last = &mut self.lasts[i];

                *diff = (*last).wrapping_add(decoder.decode_symbol(&mut self.models[i]) as u8);
                buf[i] = *diff;
                *last = *diff;
            }
        }
    }
}