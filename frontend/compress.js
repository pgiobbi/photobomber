export async function compressImage(file, maxSizeKB = 300, maxDimension = 1920) {
    try {
        if (!file.type.startsWith('image/')) {
            throw new Error('Please upload an image file.');
        }

        const img = new Image();
        const imgPromise = new Promise((resolve, reject) => {
            img.onload = () => resolve(img);
            img.onerror = () => reject(new Error('Failed to load image.'));
        });
        img.src = URL.createObjectURL(file);

        await imgPromise;

        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');

        // Calculate new dimensions
        let width = img.width;
        let height = img.height;
        if (width > height) {
            if (width > maxDimension) {
                height = Math.round((height * maxDimension) / width);
                width = maxDimension;
            }
        } else {
            if (height > maxDimension) {
                width = Math.round((width * maxDimension) / height);
                height = maxDimension;
            }
        }

        canvas.width = width;
        canvas.height = height;
        ctx.drawImage(img, 0, 0, width, height);

        // Try WebP compression first
        let quality = 85;
        let compressedDataUrl;
        let fileSizeKB;
        let format = 'image/webp';
        let qualityStep = 5;
        let minQuality = 30;

        console.log(`Starting WebP compression: target ${maxSizeKB}KB, dims ${width}x${height}`);

        do {
            compressedDataUrl = canvas.toDataURL(format, quality / 100);
            fileSizeKB = ((compressedDataUrl.length - `data:${format};base64,`.length) * 3) / 4 / 1024;
            console.log(`WebP quality ${quality}, size ${fileSizeKB.toFixed(2)}KB`);
            quality -= qualityStep;
        } while (fileSizeKB > maxSizeKB && quality > minQuality);

        // Fallback to JPEG if WebP fails to meet size target
        if (fileSizeKB > maxSizeKB) {
            console.log(`WebP failed to meet ${maxSizeKB}KB, switching to JPEG`);
            format = 'image/jpeg';
            quality = 0.9;
            minQuality = 0.2;
            qualityStep = 0.1;

            // Clear canvas and redraw to avoid memory issues
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.drawImage(img, 0, 0, width, height);

            do {
                compressedDataUrl = canvas.toDataURL(format, quality);
                fileSizeKB = ((compressedDataUrl.length - `data:${format};base64,`.length) * 3) / 4 / 1024;
                console.log(`JPEG quality ${quality}, size ${fileSizeKB.toFixed(2)}KB`);
                quality -= qualityStep;
            } while (fileSizeKB > maxSizeKB && quality > minQuality);
        }

        // Convert to Blob
        const blob = await fetch(compressedDataUrl)
            .then(res => res.blob())
            .then(blob => new Blob([blob], {type: format}));

        // Clean up
        URL.revokeObjectURL(img.src);
        ctx.clearRect(0, 0, canvas.width, canvas.height); // Explicitly clear canvas

        return {
            compressedBlob: blob,
            fileSizeKB: (blob.size / 1024).toFixed(2),
            dimensions: {width, height},
            qualityUsed: quality + qualityStep,
            formatUsed: format
        };
    } catch (error) {
        console.error('Compression error:', error);
        throw error;
    }
}
