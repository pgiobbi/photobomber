import {showModal} from './utils.js';

const galleryPopup = document.getElementById('galleryPopup');
const closeGallery = document.getElementById('closeGallery');
const galleryContainer = document.getElementById('galleryContainer');

async function fetchLeaderboardImages() {
    try {
        const response = await fetch('/api/public/images/leaderboard', {
            method: 'GET',
            headers: {'Content-Type': 'application/json'}
        });
        if (!response.ok) throw new Error('Failed to fetch leaderboard images');
        return await response.json();
    } catch (err) {
        console.error('Failed to fetch leaderboard images:', err);
        showModal('Oops!', 'Failed to load leaderboard images. Please try again!', 'Close');
        return [];
    }
}

async function populateGallery() {
    galleryContainer.innerHTML = '';
    const images = await fetchLeaderboardImages();
    if (images.length === 0) {
        galleryContainer.innerHTML = '<p class="text-white text-center">No photobombs yet! Be the first! 📸</p>';
        return;
    }
    images.forEach(image => {
        const container = document.createElement('div');
        container.className = 'image-container';
        const img = document.createElement('img');
        img.className = 'gallery-image';
        img.alt = `Photobomb ${image.id}`;
        img.dataset.src = `/api/public/images/${image.filename}`;
        img.dataset.id = image.id;
        img.dataset.karma = image.karma;
        const karmaBadge = document.createElement('div');
        karmaBadge.className = 'karma-badge';
        karmaBadge.textContent = `${image.karma}`;
        container.appendChild(karmaBadge);
        karmaBadge.addEventListener('click', async () => {
            try {
                const response = await fetch(`/api/public/images/${image.id}/upvote`, {method: 'POST'});
                if (response.ok) {
                    image.karma += 1;
                    karmaBadge.textContent = `${image.karma}`;
                    let data = await response.json();
                    if (data.rowsAffected === 1) {
                        showModal('Success!', 'Image upvoted successfully! 🎉');
                    } else {
                        throw new Error("Image could not be upvoted. Please try again!");
                    }
                } else {
                    throw new Error("Before upvoting you need to photobomb us first! 3 upvotes per photobomb! Go take a shot! 📸");
                }
            } catch (err) {
                console.error('Upvote failed:', err);
                showModal('Oops!', `Upvote failed! ${err.message}`, 'Close');
            }
        });
        const spinner = document.createElement('div');
        spinner.className = 'spinner';
        container.appendChild(img);
        container.appendChild(spinner);
        galleryContainer.appendChild(container);
    });
    setupLazyLoading();
}

function setupLazyLoading() {
    const imageContainers = document.querySelectorAll('.image-container');
    const observer = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const container = entry.target;
                const img = container.querySelector('.gallery-image');
                const spinner = container.querySelector('.spinner');
                img.src = img.dataset.src;
                img.onload = () => {
                    spinner.classList.add('hidden');
                    img.classList.add('loaded');
                };
                img.onerror = () => {
                    spinner.classList.add('hidden');
                    img.src = '/assets/placeholders/error.webp';
                    img.alt = 'Failed to load photobomb';
                };
                observer.unobserve(container);
            }
        });
    }, {
        root: galleryContainer,
        rootMargin: '0px 0px 100px 0px',
        threshold: 0.1
    });
    imageContainers.forEach(container => observer.observe(container));
}

export async function showGallery() {
    galleryPopup.classList.add('active');
    document.body.style.overflow = 'hidden';
    await populateGallery();
    closeGallery.addEventListener('click', () => {
        galleryPopup.classList.remove('active');
        document.body.style.overflow = '';
        galleryContainer.innerHTML = '';
    }, {once: true});
}
