// Check if running as PWA
function isPWA() {
    return window.matchMedia('(display-mode: standalone)').matches ||
           window.navigator.standalone === true;
}

// Only add pull-to-refresh for PWA mode
if (isPWA()) {
    let startY = 0;
    let pullDistance = 0;
    const threshold = 100;

    document.addEventListener('touchstart', (e) => {
        if (window.scrollY === 0) {
            startY = e.touches[0].clientY;
        }
    }, { passive: true });

    document.addEventListener('touchmove', (e) => {
        if (window.scrollY === 0 && startY > 0) {
            pullDistance = e.touches[0].clientY - startY;
            if (pullDistance > 0) {
                document.body.style.transform = `translateY(${Math.min(pullDistance / 2, 50)}px)`;
            }
        }
    }, { passive: true });

    document.addEventListener('touchend', () => {
        if (pullDistance > threshold) {
            window.location.reload();
        }
        document.body.style.transform = '';
        startY = 0;
        pullDistance = 0;
    }, { passive: true });
}
