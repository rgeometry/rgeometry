use base64;
use directories::ProjectDirs;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::{fmt, fs, path::PathBuf, str};

use tokio::process::Command;

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

fn hash_code(code: &String) -> String {
  let mut s = DefaultHasher::new();
  code.hash(&mut s);
  base64::encode_config(s.finish().to_le_bytes(), base64::URL_SAFE)
}

pub async fn compile(code: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
  let proj_dirs = ProjectDirs::from("com", "rgeometry", "rgeometry").unwrap();
  let cache_dir = proj_dirs.cache_dir();
  let hash = hash_code(&code);
  let target_file = cache_dir.join(hash).with_extension("wasm");
  fs::create_dir_all(cache_dir)?;
  if target_file.exists() {
    return Ok(target_file);
  }

  fs::write("playground/wasm/src/user.rs", &code)?;

  let output = Command::new("wasm-pack")
    .arg("build")
    .arg("--target=web")
    .arg("--no-typescript")
    .current_dir("playground/wasm/")
    .output()
    .await?;

  // Await until the command completes
  if output.status.success() {
    fs::copy("playground/wasm/pkg/wasm_bg.wasm", &target_file)?;
    Ok(target_file)
  } else {
    let stderr = str::from_utf8(&output.stderr)?;
    Err(Box::new(CompileError::new(stderr)))
  }
}
