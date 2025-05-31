import {secondsToHumanReadable, showModal} from './utils.js';
import {compressImage} from './compress.js';

document.addEventListener('DOMContentLoaded', () => {
    const cameraInput = document.getElementById('cameraInput');
    const bomberCount = document.getElementById('bomber-count');
    const step1 = document.getElementById('step1');
    const step2 = document.getElementById('step2');
    const step3 = document.getElementById('step3');
    const previewImage = document.getElementById('previewImage');
    const finalImage = document.getElementById('finalImage');
    const retakeBtn = document.getElementById('retakeBtn');
    const uploadBtn = document.getElementById('uploadBtn');
    const newPhotoBtn = document.getElementById('newPhotoBtn');
    const locationElement = document.getElementById('currentLocation');
    const lastUpdatedSpan = document.getElementById('lastUpdated');
    const galleryTrigger = document.getElementById('galleryTrigger');
    const strobeOverlay = document.getElementById('strobeOverlay');
    const isPublic = document.getElementById('isPublic');

    // Fetch and display current location
    async function fetchCurrentLocation() {
        try {
            const response = await fetch('/api/public/location', {
                method: 'GET',
                headers: {'Content-Type': 'application/json'}
            });
            if (!response.ok) throw new Error('Failed to fetch current location');
            const data = await response.json();
            const currentLocation = data.location;
            const updatedAgo = data.updatedAgo;
            locationElement.textContent = currentLocation?.name || 'Not set';
            // if (currentLocation?.name) locationElement.classList.add('text-yellow-300');
            lastUpdatedSpan.textContent = updatedAgo !== null && updatedAgo !== undefined
                ? secondsToHumanReadable(Math.floor(updatedAgo / 1000))
                : 'Not set';
        } catch (err) {
            console.error('Failed to fetch current location:', err);
            locationElement.textContent = 'Failed to load';
            lastUpdatedSpan.textContent = 'Failed to load';
        }
    }

    // Fetch photo bomber count
    async function fetchBomberCount() {
        try {
            const response = await fetch('/api/public/images/count');
            if (!response.ok) throw new Error('API request failed');
            const data = await response.json();
            bomberCount.textContent = data.count.toLocaleString();
        } catch (err) {
            console.error('Failed to fetch count:', err);
            bomberCount.textContent = 'N/A';
        }
    }

    // Perform login
    async function login() {
        try {
            await fetch('/api/auth/login', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: '{}'
            });
        } catch (err) {
            console.error('Failed to login:', err);
        }
    }

    // Load initial data
    login().then(() => {
        fetchBomberCount();
        fetchCurrentLocation();
    });

    // Handle gallery trigger click with dynamic import
    let leaderboardModule = null;
    galleryTrigger.addEventListener('click', async () => {
        try {
            if (!leaderboardModule) {
                leaderboardModule = await import('./leaderboard.js');
            }
            await leaderboardModule.showGallery();
        } catch (err) {
            console.error('Failed to load leaderboard:', err);
            showModal('Oops!', 'Failed to load the gallery. Please try again!', 'Close');
        }
    });

    // Retake button
    retakeBtn.addEventListener('click', () => {
        step2.classList.add('hidden');
        step1.classList.remove('hidden');
        cameraInput.value = '';
        if (isPublic) isPublic.checked = false;
    });

    // Trigger success animations
    function triggerSuccessAnimations() {
        strobeOverlay.classList.add('strobe-active');
        setTimeout(() => strobeOverlay.classList.remove('strobe-active'), 4800);
        const colors = [
            'linear-gradient(45deg, #ff00ff, #d7ef00)',
            'linear-gradient(45deg, #ff4444, #41B0E8)',
            'linear-gradient(45deg, #d7ef00, #ff00ff)'
        ];
        for (let i = 0; i < 30; i++) {
            const confetti = document.createElement('div');
            confetti.className = 'confetti';
            confetti.style.top = `-10vh`;
            confetti.style.left = `${Math.random() * 100}vw`;
            confetti.style.background = colors[Math.floor(Math.random() * colors.length)];
            confetti.style.animationDelay = `${Math.random() * 2}s`;
            document.body.appendChild(confetti);
            setTimeout(() => confetti.remove(), 4000);
        }
    }
    
    async function onUploadClicked() {
        const files = cameraInput.files;
        if (!files.length) return;
        const formData = new FormData();
        try {
            uploadBtn.textContent = "Uploading...";
            uploadBtn.disabled = true;
            const file = files[0];
            const res = await compressImage(file);
            const {compressedBlob, fileSizeKB} = res;
            if (fileSizeKB > 1024) {
                showModal('File Too Large', 'Images must be under 1MB. Try a smaller file!');
                uploadBtn.textContent = "Upload! 🚀";
                uploadBtn.disabled = false;
                return;
            }
            formData.append('images', compressedBlob);
            formData.append('isPublic', isPublic?.checked || false);
            const response = await fetch('/api/public/images/upload', {
                method: 'POST',
                body: formData,
            });
            if (!response.ok) {
                const errorData = await response.text();
                throw new Error(errorData || 'Upload failed');
            }
            await fetchBomberCount();
            finalImage.src = previewImage.src;
            step2.classList.add('hidden');
            step3.classList.remove('hidden');
            const stats = [
                "🌟 PHOTOBOMB HERO 🌟",
                "🔥 TOP BOMBER 🔥",
                "💯 FESTIVAL PRO 💯",
                "🎪 STAGE CRUSHER 🎪",
                "🌌 COSMIC PHOTOBOMBER 🌌",
                "⚡️ ELECTRIC VIBE IGNITER ⚡️",
                "🎇 FESTIVAL FUSE BLAZER 🎇",
                "🌈 RAVE LEGEND UNLEASHED 🌈",
                "🔥 PULSE POUNDING MAVERICK 🔥",
                "✨ STARDUST SCENE STEALER ✨",
                "🎉 BASSLINE TRAILBLAZER 🎉",
                "💥 BOOMDROP RENEGADE 💥",
                "🪐 ORBITAL RHYTHM RIDER 🪐",
                "🌠 METEORIC MOMENT MAKER 🌠",
                "🎶 SONIC WAVE WARRIOR 🎶",
                "⚡ VOLT-CHARGED VISIONARY ⚡",
                "🌀 NEON SPIRAL SORCERER 🌀",
                "🎡 RAVE RINGMASTER 🎡",
                "💫 GALACTIC GROOVE GURU 💫",
                "🔊 BEAT QUAKE CREATOR 🔊"
            ];
            const badge = document.querySelector('.badge');
            badge.textContent = stats[Math.floor(Math.random() * stats.length)];
            triggerSuccessAnimations();
            uploadBtn.textContent = "Upload! 🚀";
            uploadBtn.disabled = false;
        } catch (err) {
            console.error('Upload error:', err);
            showModal('Oops! 😅', `Failed to upload: ${err.message}. Try again!`, 'Retry');
            uploadBtn.textContent = "Upload! 🚀";
            uploadBtn.disabled = false;
        }
    }

    // Upload button
    uploadBtn.addEventListener('click', async () => await onUploadClicked());
    
    // Handle file input change
    cameraInput.addEventListener('change', async (event) => {
        const files = event.target.files;
        if (files.length !== 1) return;
        const allowedTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp'];
        if (!allowedTypes.includes(files[0].type)) {
            showModal('Invalid File', 'Please upload only JPG, PNG, GIF, HEIC, HEIF, or WebP images.');
            return;
        }
        const file = files[0];
        const fileURL = URL.createObjectURL(file);
        previewImage.src = fileURL;
        step1.classList.add('hidden');
        step2.classList.remove('hidden');
        onUploadClicked();
    });

    // New photo button
    newPhotoBtn.addEventListener('click', () => {
        step3.classList.add('hidden');
        step1.classList.remove('hidden');
        cameraInput.value = '';
        if (isPublic) isPublic.checked = false;
    });
});
