use base64;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::error::Error;
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
pub async fn compile(code: String) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
  fs::write("../wasm/src/user.rs", &code)?;

  let output = Command::new("wasm-pack")
    .arg("build")
    .arg("--target=web")
    .arg("--no-typescript")
    .current_dir("../wasm/")
    .output()
    .await?;

  // Await until the command completes
  if output.status.success() {
    let mut s = DefaultHasher::new();
    code.hash(&mut s);
    let key: String = base64::encode_config(s.finish().to_le_bytes(), base64::URL_SAFE);
    let mut output = PathBuf::from(key);
    output.set_extension("wasm");
    fs::copy("../wasm/pkg/wasm_bg.wasm", &output)?;
    Ok(output)
  } else {
    let stderr = str::from_utf8(&output.stderr)?;
    Err(Box::new(CompileError::new(stderr)))
  }
}
