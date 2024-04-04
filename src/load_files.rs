/// Load files from disk into memory
use walkdir::WalkDir;
use mime_guess::Mime;
use std::collections::HashMap;

use hyper::body::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use lazy_static::lazy_static;
use crate::typedef::GenericError;

pub struct FileEntry {
  pub size: u64,
  pub content: &'static [u8],
  pub mime_type: Mime,
}

/// iterate over files in dir_name
pub fn load_files(dir_name: &str) -> HashMap<String, FileEntry> {
  let mut files: HashMap<String, FileEntry> = HashMap::new();
  for entry in WalkDir::new(dir_name) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let content = std::fs::read(entry.path()).unwrap();
      let path = entry.path().to_str().unwrap()[dir_name.len().. ].to_string();
      log::info!("{}, size: {}", path, entry.metadata().unwrap().len());
      let file_entry = FileEntry {
        size: entry.metadata().unwrap().len(),
        content: Box::<[u8]>::leak(content.into_boxed_slice()),
        mime_type: mime_guess(entry.path().to_str().unwrap()),
      };
      files.insert(path, file_entry);
    }
  }
  files
}

fn mime_guess(file_path: &str) -> Mime {
  mime_guess::from_path(file_path).first_or_text_plain()
}

lazy_static! {
  pub static ref FILES: HashMap<String, FileEntry> = load_files("html");
}

pub async fn serve_file(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
    let path = match req.uri().path() {
      "/" => "/index.html",
      _ => req.uri().path(),
    };
    let file = match FILES.get(path) {
      Some(file) => file,
      None => {
        return Ok(Response::builder()
          .status(404)
          .body(Full::new(Bytes::from_static(b"Not Found")))
          .unwrap());
      }
    };
    log::info!("mime_type: {}", file.mime_type);
    Ok(Response::builder()
      .status(200)
      .header("Content-Type", if file.mime_type.essence_str() == "text/html" {
        "text/html; charset=utf-8"
      } else {
        file.mime_type.essence_str()
      })
      .body(Full::new(Bytes::from_static(file.content)))?)
}
