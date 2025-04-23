async function compressImage(file, maxSizeKB = 300, maxDimension = 1920) {
    try {
        if (!file.type.startsWith('image/')) {
            throw new Error('Please upload an image file.');
        }
        // TODO: skip compression for .HEIC files 

        const img = new Image();
        const imgPromise = new Promise((resolve, reject) => {
            img.onload = () => resolve(img);
            img.onerror = () => reject(new Error('Failed to load image.'));
        });
        img.src = URL.createObjectURL(file);

        await imgPromise;

        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');

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

        let quality = 85; // WebP quality (0–100 scale, 85 is a good starting point)
        let compressedDataUrl;
        let fileSizeKB;

        // Iteratively compress until file size is under maxSizeKB
        do {
            compressedDataUrl = canvas.toDataURL('image/webp', quality / 100);
            fileSizeKB = ((compressedDataUrl.length - 'data:image/webp;base64,'.length) * 3) / 4 / 1024;
            quality -= 5; // Reduce quality incrementally (WebP uses 0–100 scale)
        } while (fileSizeKB > maxSizeKB && quality > 50); // Stop at 50 to avoid noticeable quality loss

        const blob = await fetch(compressedDataUrl)
            .then(res => res.blob())
            .then(blob => new Blob([blob], {type: 'image/webp'}));

        URL.revokeObjectURL(img.src);

        return {
            compressedBlob: blob,
            fileSizeKB: (blob.size / 1024).toFixed(2),
            dimensions: {width, height},
            qualityUsed: quality + 5
        };
    } catch (error) {
        console.error('Compression error:', error);
        throw error;
    }
}
