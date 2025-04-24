pub const APP_NAME: &str = "photobomber-api";

pub const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB in bytes

// Whitelist of allowed image extensions
pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

pub const ADMIN_ID: i64 = 1;
pub const PUBLIC_ID: i64 = 0;
