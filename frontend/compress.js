// Light, quality-first image preparation for upload.
//
// Goal: preserve wedding memories. We only re-encode when we have to (image too
// large in bytes or in pixels). When a photo is already a reasonable size and
// format we pass the original file through untouched, with zero quality loss.

const PASSTHROUGH_TYPES = ['image/jpeg', 'image/png', 'image/webp'];

function loadImage(file) {
    return new Promise((resolve, reject) => {
        const img = new Image();
        const url = URL.createObjectURL(file);
        img.onload = () => resolve({img, url});
        img.onerror = () => {
            URL.revokeObjectURL(url);
            reject(new Error('This image could not be read on your device.'));
        };
        img.src = url;
    });
}

function scaleToFit(width, height, maxDimension) {
    if (width <= maxDimension && height <= maxDimension) return {width, height};
    if (width >= height) {
        return {width: maxDimension, height: Math.round((height * maxDimension) / width)};
    }
    return {width: Math.round((width * maxDimension) / height), height: maxDimension};
}

function canvasToBlob(canvas, type, quality) {
    return new Promise((resolve) => canvas.toBlob(resolve, type, quality));
}

/**
 * Prepare a file for upload, keeping quality as high as possible.
 *
 * @returns {Promise<{blob: Blob, fileSizeKB: number, format: string, passthrough: boolean}>}
 */
export async function compressImage(file, options = {}) {
    const {
        maxSizeKB = 3000,     // re-encode only above this byte size
        maxDimension = 2880,  // re-encode only above this pixel dimension
        quality = 0.92,       // starting quality for re-encode (visually near-lossless)
        minQuality = 0.8,     // never degrade below this
    } = options;

    if (!file.type.startsWith('image/')) {
        throw new Error('Please choose an image file.');
    }

    const {img, url} = await loadImage(file);

    try {
        const withinSize = file.size / 1024 <= maxSizeKB;
        const withinDimensions = img.width <= maxDimension && img.height <= maxDimension;

        // Already small enough and in a web-friendly format: keep the original bytes.
        if (PASSTHROUGH_TYPES.includes(file.type) && withinSize && withinDimensions) {
            return {
                blob: file,
                fileSizeKB: Number((file.size / 1024).toFixed(2)),
                format: file.type,
                passthrough: true,
            };
        }

        const {width, height} = scaleToFit(img.width, img.height, maxDimension);
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(img, 0, 0, width, height);

        // Prefer WebP (great quality-to-size). Step quality down only if needed.
        let format = 'image/webp';
        let q = quality;
        let blob = await canvasToBlob(canvas, format, q);
        while (blob && blob.type === format && blob.size / 1024 > maxSizeKB && q > minQuality) {
            q = Number((q - 0.04).toFixed(2));
            blob = await canvasToBlob(canvas, format, q);
        }

        // Fallback to JPEG if the browser produced no WebP (rare on modern devices).
        if (!blob || blob.type !== 'image/webp') {
            format = 'image/jpeg';
            blob = await canvasToBlob(canvas, format, Math.max(q, 0.85));
        }

        if (!blob) throw new Error('Could not process this image.');

        return {
            blob,
            fileSizeKB: Number((blob.size / 1024).toFixed(2)),
            format,
            passthrough: false,
        };
    } finally {
        URL.revokeObjectURL(url);
    }
}
