pub const APP_NAME: &str = "photobomber-api";

pub const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB in bytes

// Whitelist of allowed image extensions
pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

pub const ADMIN_ID: i64 = 1;
pub const PUBLIC_ID: i64 = 0;

/// Number of images that can be upvoted for each uploaded image.
pub const UPVOTES_PER_UPLOAD: i8 = 3;