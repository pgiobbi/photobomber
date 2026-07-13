export function secondsToHumanReadable(seconds) {
    if (seconds < 60) return `${seconds} seconds ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`;
    const days = Math.floor(hours / 24);
    return `${days} day${days === 1 ? '' : 's'} ago`;
}

export function showModal(title, message, buttonText = 'OK', imageUrls = []) {
    const existingModal = document.querySelector('.modal');
    if (existingModal) existingModal.remove();

    const modal = document.createElement('div');
    modal.className = 'modal';

    const content = document.createElement('div');
    content.className = 'modal-content';

    const titleEl = document.createElement('h2');
    titleEl.textContent = title;

    const messageEl = document.createElement('p');
    messageEl.innerHTML = message;

    content.appendChild(titleEl);
    content.appendChild(messageEl);

    if (imageUrls.length > 0) {
        const thumbs = document.createElement('div');
        thumbs.className = 'modal-thumbs';
        imageUrls.forEach((url) => {
            const img = document.createElement('img');
            img.src = url;
            img.alt = 'Shared photo';
            thumbs.appendChild(img);
        });
        content.appendChild(thumbs);
    }

    const button = document.createElement('button');
    button.className = 'btn btn-primary';
    button.textContent = buttonText;
    button.addEventListener('click', () => modal.remove());

    content.appendChild(button);
    modal.appendChild(content);
    document.body.appendChild(modal);
}
