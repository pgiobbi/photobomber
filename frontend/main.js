import {showModal} from './utils.js';
import {compressImage} from './compress.js';

document.addEventListener('DOMContentLoaded', () => {
    const cameraInput = document.getElementById('cameraInput');
    const galleryInput = document.getElementById('galleryInput');
    const stepHome = document.getElementById('stepHome');
    const stepSending = document.getElementById('stepSending');
    const stepSuccess = document.getElementById('stepSuccess');
    const photoCount = document.getElementById('photoCount');
    const uploadProgress = document.getElementById('uploadProgress');
    const successMessage = document.getElementById('successMessage');
    const againBtn = document.getElementById('againBtn');

    let displayCount = 0;
    let countRaf = null;

    // --- Screen switching ---------------------------------------------------

    function showStep(step) {
        [stepHome, stepSending, stepSuccess].forEach((el) => el.classList.add('hidden'));
        step.classList.remove('hidden');
    }

    // --- Hype ticker (odometer count-up) ------------------------------------

    function animateCountTo(target) {
        if (typeof target !== 'number' || target < 0) return;
        cancelAnimationFrame(countRaf);
        const from = displayCount;
        const start = performance.now();
        const dur = 1100;
        const step = (now) => {
            const t = Math.min(1, (now - start) / dur);
            const eased = 1 - Math.pow(1 - t, 3);
            displayCount = Math.round(from + (target - from) * eased);
            photoCount.textContent = displayCount.toLocaleString();
            if (t < 1) countRaf = requestAnimationFrame(step);
        };
        countRaf = requestAnimationFrame(step);
    }

    // --- Confetti -----------------------------------------------------------

    function celebrate() {
        const colors = ['#F2C879', '#F8D48A', '#38E1D6', '#FF3DA6', '#C4652F'];
        for (let i = 0; i < 42; i++) {
            const piece = document.createElement('div');
            piece.className = 'confetti';
            piece.style.left = `${Math.random() * 100}vw`;
            piece.style.background = colors[i % colors.length];
            piece.style.transform = `rotate(${Math.random() * 360}deg)`;
            piece.style.animationDuration = `${2.4 + Math.random() * 1.8}s`;
            piece.style.animationDelay = `${Math.random() * 1.2}s`;
            document.body.appendChild(piece);
            setTimeout(() => piece.remove(), 5200);
        }
    }

    // --- Backend calls ------------------------------------------------------

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
            animateCountTo(data.count);
        } catch (err) {
            console.error('Failed to fetch count:', err);
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

    // --- Collapsed flow: pick -> send immediately -> success ----------------

    function handleSelection(fileList) {
        const files = Array.from(fileList).filter((f) => f.type.startsWith('image/'));
        cameraInput.value = '';
        galleryInput.value = '';
        if (!files.length) {
            showModal('No photos found', 'Please choose one or more image files.', 'OK');
            return;
        }
        uploadAll(files);
    }

    async function uploadAll(files) {
        showStep(stepSending);
        const total = files.length;
        uploadProgress.textContent = total > 1 ? `Photo 1 of ${total}` : 'Preparing\u2026';

        let succeeded = 0;
        const failedFiles = [];

        for (let i = 0; i < total; i++) {
            if (total > 1) uploadProgress.textContent = `Photo ${i + 1} of ${total}`;
            const file = files[i];
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

        await fetchPhotoCount();

        const failed = failedFiles.length;

        // All failed: back to home with a retry prompt.
        if (succeeded === 0) {
            showStep(stepHome);
            showModal(
                'Something went wrong',
                'We could not send your photo - festival network, right? Check your connection and tap "TAKE THE PHOTO!" to try again.',
                'Try again',
            );
            return;
        }

        const plural = succeeded === 1 ? 'photobomb' : 'photobombs';
        successMessage.textContent = `${succeeded} ${plural} delivered straight to our phones!`;

        showStep(stepSuccess);
        celebrate();

        // Some failed: let them know, but still celebrate the ones that made it.
        if (failed > 0) {
            const fp = failed === 1 ? 'photo' : 'photos';
            showModal(
                'Almost there',
                `${succeeded} made it! ${failed} ${fp} could not be sent - tap "BOMB US AGAIN!" to retry.`,
                'Got it',
            );
        }
    }

    // --- Wiring -------------------------------------------------------------

    cameraInput.addEventListener('change', (e) => handleSelection(e.target.files));
    galleryInput.addEventListener('change', (e) => handleSelection(e.target.files));
    againBtn.addEventListener('click', () => showStep(stepHome));

    // Initial load
    login().then(fetchPhotoCount);
    // Keep the ticker feeling live while on the home screen.
    setInterval(() => {
        if (!stepHome.classList.contains('hidden')) fetchPhotoCount();
    }, 15000);
});
