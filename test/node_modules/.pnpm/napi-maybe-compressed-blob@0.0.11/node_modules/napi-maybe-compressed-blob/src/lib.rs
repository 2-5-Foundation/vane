#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;

pub const CODE_BLOB_BOMB_LIMIT: usize = 50 * 1024 * 1024;
pub type WasmBytes = Vec<u8>;

#[napi]
pub fn compress(buf: Buffer) -> Result<Buffer> {
  match sp_maybe_compressed_blob::compress(&buf, CODE_BLOB_BOMB_LIMIT) {
    Some(bytes_compressed) => {
      Ok(Buffer::from(bytes_compressed.to_vec()))
    },
    None => Err(Error::new(napi::Status::GenericFailure,String::from("Compression failed and returned nothing")))
  }
}

#[napi]
pub fn decompress(buf: Buffer) -> Result<Buffer> {
  match sp_maybe_compressed_blob::decompress(&buf, CODE_BLOB_BOMB_LIMIT)
    .map(|res| res.to_vec())
    .map_err(|err| err.to_string()) {
      Ok(bytes_decompressed) => {
        Ok(Buffer::from(bytes_decompressed.to_vec()))
      },
      Err(_) => Err(Error::new(napi::Status::GenericFailure,String::from("Decompression failed")))
    }
}