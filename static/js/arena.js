// Comportements propres à la disposition Arena (sans effet sur les autres dispositions).
function initReel() {
    const reel = document.getElementById('ar-reel');
    if (!reel) return;

    const step = () => {
        const first = reel.querySelector('.ar-poster');
        return first ? first.getBoundingClientRect().width + 16 : reel.clientWidth;
    };

    const prev = document.getElementById('ar-reel-prev');
    const next = document.getElementById('ar-reel-next');
    if (prev) prev.addEventListener('click', () => reel.scrollBy({ left: -step(), behavior: 'smooth' }));
    if (next) next.addEventListener('click', () => reel.scrollBy({ left: step(), behavior: 'smooth' }));
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initReel);
} else {
    initReel();
}
