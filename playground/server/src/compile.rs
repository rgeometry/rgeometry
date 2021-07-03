use directories::ProjectDirs;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::{fmt, fs, str};

use tokio::process::Command;

const GIT_VERSION: &str = git_version::git_version!();

#[derive(Debug)]
struct CompileError(String);

impl CompileError {
  fn new(msg: &str) -> CompileError {
    CompileError(msg.to_string())
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

pub async fn compile(mut code: String) -> Result<String, Box<dyn Error + Send + Sync>> {
  code += "\nmod support;\n";
  let proj_dirs = ProjectDirs::from("com", "rgeometry", "rgeometry").unwrap();
  let cache_dir = proj_dirs.cache_dir();
  let hash = hash_code(&code);
  let html_file = cache_dir.join(&hash).with_extension("html");
  fs::create_dir_all(cache_dir)?;
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
      Err(Box::new(CompileError::new("Internal error: Failed to bundle wasm module.")))
    }
  } else {
    let stderr = str::from_utf8(&output.stderr)?;
    Err(Box::new(CompileError::new(stderr)))
  }
}
