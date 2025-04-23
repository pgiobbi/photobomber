use std::path::Path;
use mime::Mime;

// Map MIME type to file extension
pub fn get_extension_from_mime(mime: &Mime) -> &str {
    match mime.subtype().as_str() {
        "jpeg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "bmp" => "bmp",
        "webp" => "webp",
        _ => "jpg", // Default to jpg for unknown image types
    }
}

// Extract extension from filename
pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(|ext| ext.to_str())
}
