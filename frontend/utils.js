export function secondsToHumanReadable(seconds) {
    if (seconds < 60) return `${seconds} seconds ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`;
    const days = Math.floor(hours / 24);
    return `${days} day${days === 1 ? '' : 's'} ago`;
}

export function showModal(title, message, buttonText = 'Keep Dancing! 🕺', imageUrls = []) {
    const existingModal = document.querySelector('.modal');
    if (existingModal) document.body.removeChild(existingModal);
    const modal = document.createElement('div');
    modal.className = 'modal fixed inset-0 bg-black bg-opacity-80 flex items-center justify-center z-50';
    const modalContent = document.createElement('div');
    modalContent.className = 'bg-gray-900 bg-opacity-90 p-6 rounded-xl max-w-sm w-full mx-4 text-center';
    const titleEl = document.createElement('h2');
    titleEl.className = 'text-2xl font-bold mb-4 text-white glow';
    titleEl.textContent = title;
    const messageEl = document.createElement('p');
    messageEl.className = 'mb-6 text-white';
    messageEl.innerHTML = message;
    if (imageUrls.length > 0) {
        const previewContainer = document.createElement('div');
        previewContainer.className = 'flex flex-wrap justify-center gap-2 mb-4';
        imageUrls.forEach(url => {
            const img = document.createElement('img');
            img.src = url;
            img.className = 'w-16 h-16 object-cover rounded';
            img.alt = 'Uploaded photobomb';
            previewContainer.appendChild(img);
        });
        modalContent.appendChild(previewContainer);
    }
    const button = document.createElement('button');
    button.className = 'gradient-animate text-white font-bold py-3 px-6 rounded-full';
    button.textContent = buttonText;
    button.addEventListener('click', () => {
        document.body.removeChild(modal);
    });
    modalContent.appendChild(titleEl);
    modalContent.appendChild(messageEl);
    modalContent.appendChild(button);
    modal.appendChild(modalContent);
    document.body.appendChild(modal);
}
