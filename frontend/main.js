import {showModal} from './utils.js';
import {compressImage} from './compress.js';

document.addEventListener('DOMContentLoaded', () => {
    const cameraInput = document.getElementById('cameraInput');
    const galleryInput = document.getElementById('galleryInput');
    const step1 = document.getElementById('step1');
    const step2 = document.getElementById('step2');
    const step3 = document.getElementById('step3');
    const previewGrid = document.getElementById('previewGrid');
    const selectedCount = document.getElementById('selectedCount');
    const uploadBtn = document.getElementById('uploadBtn');
    const cancelBtn = document.getElementById('cancelBtn');
    const addMoreBtn = document.getElementById('addMoreBtn');
    const photoCount = document.getElementById('photoCount');
    const successMessage = document.getElementById('successMessage');

    // Files chosen for the current upload, plus their object URLs (for cleanup).
    let selectedFiles = [];
    let objectUrls = [];

    // --- Helpers ------------------------------------------------------------

    function showStep(step) {
        [step1, step2, step3].forEach((el) => el.classList.add('hidden'));
        step.classList.remove('hidden');
    }

    function revokeObjectUrls() {
        objectUrls.forEach((url) => URL.revokeObjectURL(url));
        objectUrls = [];
    }

    function resetToStart() {
        revokeObjectUrls();
        selectedFiles = [];
        previewGrid.innerHTML = '';
        cameraInput.value = '';
        galleryInput.value = '';
        uploadBtn.disabled = false;
        uploadBtn.textContent = 'Send it';
        showStep(step1);
    }

    function createSpinner() {
        const spinner = document.createElement('div');
        spinner.id = 'uploadSpinner';
        spinner.className = 'upload-spinner';
        spinner.innerHTML = `
            <div class="ring"></div>
            <div class="loading-text">Dropping your photobomb</div>
            <div class="loading-subtext" id="uploadProgress">Preparing...</div>
        `;
        document.body.appendChild(spinner);
        requestAnimationFrame(() => spinner.classList.add('active'));
    }

    function updateProgress(current, total) {
        const el = document.getElementById('uploadProgress');
        if (el) el.textContent = `Photo ${current} of ${total}`;
    }

    function removeSpinner() {
        const spinner = document.getElementById('uploadSpinner');
        if (!spinner) return;
        spinner.classList.remove('active');
        setTimeout(() => spinner.remove(), 300);
    }

    function celebrate() {
        const colors = ['#D9A648', '#F2C879', '#C4652F', '#4E8E8A'];
        for (let i = 0; i < 26; i++) {
            const petal = document.createElement('div');
            petal.className = 'celebrate';
            petal.style.left = `${Math.random() * 100}vw`;
            petal.style.top = '-5vh';
            petal.style.background = colors[Math.floor(Math.random() * colors.length)];
            petal.style.animationDelay = `${Math.random() * 1.2}s`;
            petal.style.opacity = '0.8';
            document.body.appendChild(petal);
            setTimeout(() => petal.remove(), 5000);
        }
    }

    // --- Backend calls ------------------------------------------------------

    // Obtain a public session cookie so uploads are authorized.
    async function login() {
        try {
            await fetch('/api/auth/login', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: '{}',
            });
        } catch (err) {
            console.error('Failed to login:', err);
        }
    }

    async function fetchPhotoCount() {
        try {
            const response = await fetch('/api/public/images/count');
            if (!response.ok) throw new Error('API request failed');
            const data = await response.json();
            photoCount.textContent = data.count.toLocaleString();
        } catch (err) {
            console.error('Failed to fetch count:', err);
            photoCount.textContent = '--';
        }
    }

    function extensionFor(blob, fallbackName) {
        if (blob.type === 'image/webp') return 'webp';
        if (blob.type === 'image/png') return 'png';
        if (blob.type === 'image/gif') return 'gif';
        if (blob.type === 'image/jpeg') return 'jpg';
        const match = (fallbackName || '').match(/\.([a-z0-9]+)$/i);
        return match ? match[1].toLowerCase() : 'jpg';
    }

    // --- Selection flow -----------------------------------------------------

    // Render the preview grid for a set of files and move to the review step.
    // Used both for a fresh selection and when re-presenting photos that failed to upload.
    function renderSelection(files, countLabel) {
        revokeObjectUrls();
        selectedFiles = files;
        previewGrid.innerHTML = '';

        files.forEach((file) => {
            const url = URL.createObjectURL(file);
            objectUrls.push(url);
            const thumb = document.createElement('div');
            thumb.className = 'thumb';
            const img = document.createElement('img');
            img.src = url;
            img.alt = 'Selected photo';
            thumb.appendChild(img);
            previewGrid.appendChild(thumb);
        });

        const n = files.length;
        selectedCount.textContent = countLabel || `${n} photo${n === 1 ? '' : 's'} selected`;
        showStep(step2);
    }

    function handleSelection(fileList) {
        const files = Array.from(fileList).filter((f) => f.type.startsWith('image/'));
        if (!files.length) {
            showModal('No photos found', 'Please choose one or more image files.', 'OK');
            return;
        }
        renderSelection(files);
    }

    async function uploadAll() {
        if (!selectedFiles.length) return;

        uploadBtn.disabled = true;
        uploadBtn.textContent = 'Sending...';
        createSpinner();

        let succeeded = 0;
        const failedFiles = [];
        const total = selectedFiles.length;
        // Snapshot the list: renderSelection (on the retry path) reassigns selectedFiles.
        const filesToUpload = selectedFiles.slice();

        for (let i = 0; i < total; i++) {
            updateProgress(i + 1, total);
            const file = filesToUpload[i];
            try {
                const {blob} = await compressImage(file);
                const ext = extensionFor(blob, file.name);
                const formData = new FormData();
                formData.append('images', blob, `photo.${ext}`);
                formData.append('isPublic', 'false');

                const response = await fetch('/api/public/images/upload', {
                    method: 'POST',
                    body: formData,
                });
                if (!response.ok) {
                    const errorText = await response.text();
                    throw new Error(errorText || 'Upload failed');
                }
                succeeded++;
            } catch (err) {
                console.error(`Failed to upload photo ${i + 1}:`, err);
                failedFiles.push(file);
            }
        }

        const failed = failedFiles.length;

        removeSpinner();
        await fetchPhotoCount();

        uploadBtn.disabled = false;
        uploadBtn.textContent = 'Send it';

        // Any failures: keep the failed photos selected so the guest can retry just those.
        if (failed > 0) {
            const fp = failed === 1 ? 'photo' : 'photos';
            renderSelection(failedFiles, `${failed} ${fp} to retry`);
            cameraInput.value = '';
            galleryInput.value = '';

            if (succeeded === 0) {
                showModal(
                    'Something went wrong',
                    'We could not send your photos. They are still here - festival network, right? Check your connection and tap "Send it" to try again.',
                    'Try again',
                );
            } else {
                const sp = succeeded === 1 ? 'photo' : 'photos';
                showModal(
                    'Almost there',
                    `${succeeded} ${sp} made it to our phones. ${failed} ${fp} could not be sent - they are still here, tap "Send it" to try again.`,
                    'Retry',
                );
            }
            return;
        }

        const plural = succeeded === 1 ? 'photobomb' : 'photobombs';
        successMessage.textContent = `${succeeded} ${plural} delivered to our phones.`;

        revokeObjectUrls();
        selectedFiles = [];
        previewGrid.innerHTML = '';
        cameraInput.value = '';
        galleryInput.value = '';

        showStep(step3);
        celebrate();
    }

    // --- Wiring -------------------------------------------------------------

    cameraInput.addEventListener('change', (e) => handleSelection(e.target.files));
    galleryInput.addEventListener('change', (e) => handleSelection(e.target.files));
    uploadBtn.addEventListener('click', uploadAll);
    cancelBtn.addEventListener('click', resetToStart);
    addMoreBtn.addEventListener('click', resetToStart);

    // Initial load
    login().then(fetchPhotoCount);
});
