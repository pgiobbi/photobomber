// Aggressive, bandwidth-first image preparation for upload.
//
// Goal: get the photo through the festival network. Cell coverage at the venue
// is saturated, so we compress hard on the device: downscale to a phone-friendly
// resolution and re-encode to WebP at moderate quality. Originals are passed
// through only when they are already tiny.

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
 * Prepare a file for upload, keeping the payload as small as possible.
 *
 * @returns {Promise<{blob: Blob, fileSizeKB: number, format: string, passthrough: boolean}>}
 */
export async function compressImage(file, options = {}) {
    const {
        maxSizeKB = 350,      // target upload size; step quality down until under this
        maxDimension = 1600,  // plenty for phone screens and small prints
        quality = 0.75,       // starting quality for re-encode
        minQuality = 0.5,     // never degrade below this
    } = options;

    if (!file.type.startsWith('image/')) {
        throw new Error('Please choose an image file.');
    }

    const {img, url} = await loadImage(file);

    try {
        const withinSize = file.size / 1024 <= maxSizeKB;
        const withinDimensions = img.width <= maxDimension && img.height <= maxDimension;

        // Already tiny and in a web-friendly format: keep the original bytes.
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
            q = quality;
            blob = await canvasToBlob(canvas, format, q);
            while (blob && blob.size / 1024 > maxSizeKB && q > minQuality) {
                q = Number((q - 0.05).toFixed(2));
                blob = await canvasToBlob(canvas, format, q);
            }
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
