use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{fmt, fs, str};

use rocket::*;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

const GIT_VERSION: &str = git_version::git_version!();

#[derive(Responder, Debug)]
#[response(status = 200, content_type = "plain")]
pub struct CompileError(String);

impl CompileError {
  fn new(msg: &str) -> CompileError {
    CompileError(msg.to_string())
  }
}

// impl From<CompileError> for String {
//   fn from(error: CompileError) -> String {
//     error.0
//   }
// }

impl From<std::io::Error> for CompileError {
  fn from(error: std::io::Error) -> CompileError {
    CompileError(error.to_string())
  }
}

impl From<std::str::Utf8Error> for CompileError {
  fn from(error: std::str::Utf8Error) -> CompileError {
    CompileError(error.to_string())
  }
}

impl From<reqwest::Error> for CompileError {
  fn from(error: reqwest::Error) -> CompileError {
    CompileError(error.to_string())
  }
}

impl fmt::Display for CompileError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Error for CompileError {
  fn description(&self) -> &str {
    &self.0
  }
}

fn hash_code(code: &str) -> String {
  let mut s = DefaultHasher::new();
  code.hash(&mut s);
  GIT_VERSION.hash(&mut s);
  base64::encode_config(s.finish().to_le_bytes(), base64::URL_SAFE)
}

pub fn get_cache_dir() -> PathBuf {
  ProjectDirs::from("com", "rgeometry", "rgeometry")
    .unwrap()
    .cache_dir()
    .to_owned()
}

pub async fn compile(mut code: String) -> Result<String, CompileError> {
  static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
  let _guard: MutexGuard<'_, ()> = LOCK.lock().await;
  code += "\nmod support;\n";
  code += "use rgeometry::{data::*, *};\n";
  code += "use rgeometry_wasm::playground::*;\n"
  let cache_dir = get_cache_dir();
  fs::create_dir_all(&cache_dir)?;
  let hash = hash_code(&code);
  let html_file = cache_dir.join(&hash).with_extension("html");
  if html_file.exists() {
    return Ok(hash);
  }

  fs::write("playground/wasm/src/lib.rs", &code)?;

  let output = Command::new("wasm-pack")
    .arg("build")
    .arg("--target=no-modules")
    .arg("--no-typescript")
    .current_dir("playground/wasm/")
    .output()
    .await?;

  // Await until the command completes
  if output.status.success() {
    let status = Command::new("cargo")
      .arg("run")
      .current_dir("playground/wasm/")
      .status()
      .await?;

    if status.success() {
      fs::copy("playground/wasm/rgeometry-wasm.html", &html_file)?;
      Ok(hash)
    } else {
      Err(CompileError::new(
        "Internal error: Failed to bundle wasm module.",
      ))
    }
  } else {
    let stderr = str::from_utf8(&output.stderr)?;
    Err(CompileError::new(stderr))
  }
}
