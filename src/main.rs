use std::{fs, path::Path};
use tiny_http::{Server, Response, Request, Header};

fn main() {
    env_logger::init();
    let server = Server::http("127.0.0.1:8080").unwrap();
    let base_directory = Path::new("web/");

    log::info!("Serving files from '{base_directory:?}' on 127.0.0.1:8080...");

    for request in server.incoming_requests() {
        handle_request(base_directory, request);
    }
}

const SERVED_FILES: &[&str] = &[
    "index.html",
    "new-api/pkg/new_api_bg.wasm",
    "new-api/pkg/new_api_bg.wasm.d.ts",
    "new-api/pkg/new_api.d.ts",
    "new-api/pkg/new_api.js",
];

fn handle_request(base_path: &Path, request: Request) {
    let url = request.url().split('?').next().unwrap_or("/");

    let (file_path, allowed) = if url == "/" {
        log::info!("Received request for '/' returning 'index.html'");
        (base_path.join("index.html"), true)
    } else {
        log::info!("Received request for '{url}'");
        let path = url.trim_start_matches('/');
        (base_path.join(path), SERVED_FILES.contains(&path))
    };

    if allowed && Path::new(&file_path).exists() && file_path.is_file() {
        match fs::read(&file_path) {
            Ok(contents) => {
                let mime_type = get_mime_type(&file_path);
                let content_header = Header::from_bytes(b"Content-Type", mime_type).unwrap();
                let response = Response::from_data(contents).with_header(content_header);

                log::info!("File succefully found and being returned.");
                request.respond(response).unwrap();
            }
            Err(_) => {
                let response = Response::from_string("500 Internal Server Error").with_status_code(500);
                log::error!("Bad request returning 500 internal server error");
                request.respond(response).unwrap();
            }
        }
    } else {
        let response = Response::from_string("404 Not Found").with_status_code(404);
        log::error!("Requested a file that we cannot find. Returning 404.");
        request.respond(response).unwrap();
    }
}

fn get_mime_type(file_path: &Path) -> &'static [u8] {
    match file_path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => b"text/html",
        Some("css") => b"text/css",
        Some("js") => b"application/javascript",
        Some("json") => b"application/json",
        Some("wasm") => b"application/wasm",
        Some("png") => b"image/png",
        Some("jpg") | Some("jpeg") => b"image/jpeg",
        Some("gif") => b"image/gif",
        Some("svg") => b"image/svg+xml",
        _ => b"application/octet-stream", // Default binary stream
    }
}
