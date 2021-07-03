// accept ws connection
// read program source
// iff has wasm, return wasm hash code
// else:
// aquire lock
// write program to playground/wasm/lib/user.rs
// compile
// copy wasm file to ~/.cache/rgeometry/[hash].wasm
// release lock
// send response with the hash of the file.

use std::{sync::Arc};

mod manager;
use manager::Manager;

mod compile;
use compile::{compile, CompileError};

use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

use rocket::*;
use rocket::fs::NamedFile;

static GIST_CACHE: Lazy<RwLock<HashMap<String,String>>> = Lazy::new(|| {
  RwLock::new(HashMap::new())
});

fn query_cache(gist: &str) -> Option<String> {
  GIST_CACHE.read().unwrap().get(gist).cloned()
}

fn update_cache(gist: &str, code: &str) {
  let mut cache = GIST_CACHE.write().unwrap();
  cache.insert(gist.to_string(), code.to_string());
}

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  Arc::new(Manager::new(compile))
});


type Compiler = Manager<String, Result<PathBuf, CompileError>>;

#[get("/?<code>")]
async fn compile_code(code: String) -> Result<NamedFile,CompileError> {
    let path = COMPILER.run(code).await?;
    Ok(NamedFile::open(path).await?)
}

async fn fetch_gist(gist: &str) -> Result<String, reqwest::Error> {
  let code = reqwest::get(format!("https://gist.github.com/raw/{}", gist))
          .await?
          .error_for_status()?
          .text()
          .await?;
  eprintln!("Updating cache: {}", &code);
  update_cache(&gist, &code);
  Ok(code)
}

#[get("/gist/<gist>")]
async fn compile_gist(gist: String) -> Result<NamedFile, CompileError> {
  let code = match query_cache(&gist) {
    // Cache hit. Return it immediately and update the cache in the background.
    Some(code) => {
      eprintln!("Cache hit: {}", &code);
      tokio::spawn(async move { fetch_gist(&gist).await });
      code
    },
    // Cache miss. Block until we can request it from github.
    None => fetch_gist(&gist).await?
  };
  let path = COMPILER.run(code).await?;
  Ok(NamedFile::open(path).await?)
}

#[launch]
async fn rocket() -> _ {
  rocket::build()
    .attach(rocket::shield::Shield::new())
    .mount("/", routes![compile_code])
    .mount("/", routes![compile_gist])
}
