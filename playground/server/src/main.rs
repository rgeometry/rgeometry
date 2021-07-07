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

mod compile;
use compile::{compile, get_cache_dir, CompileError};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::response::Redirect;
use rocket::*;

static GIST_CACHE: Lazy<RwLock<HashMap<String, String>>> =
  Lazy::new(|| RwLock::new(HashMap::new()));

fn query_cache(gist: &str) -> Option<String> {
  GIST_CACHE.read().unwrap().get(gist).cloned()
}

fn update_cache(gist: &str, code: &str) {
  let mut cache = GIST_CACHE.write().unwrap();
  cache.insert(gist.to_string(), code.to_string());
}

#[get("/?<code>")]
async fn compile_code(code: String) -> Result<Redirect, CompileError> {
  let key = compile(code).await?;
  // Ok(NamedFile::open(path).await?)
  Ok(Redirect::to(format!("/wasm/{}.html", key)))
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
async fn compile_gist(gist: String) -> Result<Redirect, CompileError> {
  let code = match query_cache(&gist) {
    // Cache hit. Return it immediately and update the cache in the background.
    Some(code) => {
      eprintln!("Cache hit: {}", &code);
      tokio::spawn(async move { fetch_gist(&gist).await });
      code
    }
    // Cache miss. Block until we can request it from github.
    None => fetch_gist(&gist).await?,
  };
  let key = compile(code).await?;
  // Ok(NamedFile::open(path).await?)
  Ok(Redirect::to(format!("/wasm/{}.html", key)))
}

struct StaticFile(NamedFile);

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for StaticFile {
  fn respond_to(self, req: &'r Request) -> response::Result<'o> {
    Response::build_from(self.0.respond_to(req)?)
      .raw_header("Cache-control", "public, max-age=31536000, immutable") //  1 year
      .header(ContentType::HTML)
      .ok()
  }
}

#[get("/<path>")]
async fn serve_static(path: PathBuf) -> Option<StaticFile> {
  NamedFile::open(get_cache_dir().join(&path))
    .await
    .ok()
    .map(StaticFile)
}

#[launch]
async fn rocket() -> _ {
  rocket::build()
    .attach(rocket::shield::Shield::new())
    .mount("/", routes![compile_code])
    .mount("/", routes![compile_gist])
    .mount("/", routes![serve_static])
}
