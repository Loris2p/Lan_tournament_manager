import { wsClient } from './ws.js';

document.addEventListener('DOMContentLoaded', () => {
    wsClient.connect();
    initGamesCarousel();
    initTournamentCountdown();

    wsClient.on('TOURNAMENT_SCHEDULE_UPDATE', () => {
        window.location.reload();
    });
});

function initTournamentCountdown() {
    const kicker = document.getElementById('tournament-kicker');
    const textEl = document.getElementById('tournament-kicker-text');
    const dotEl = document.getElementById('tournament-kicker-dot');
    if (!kicker || !textEl) return;

    const startStr = kicker.dataset.start;
    const endStr = kicker.dataset.end;
    const isPaused = kicker.dataset.paused === 'true';
    const gamesCount = parseInt(kicker.dataset.games || '0', 10);
    const gamesSuffix = ` · ${gamesCount} ÉPREUVE${gamesCount > 1 ? 'S' : ''}`;

    const setDotStatus = (color, glow = true, pulse = true) => {
        if (!dotEl) return;
        dotEl.style.backgroundColor = color;
        dotEl.style.boxShadow = glow ? `0 0 8px ${color}` : 'none';
        dotEl.style.animation = pulse ? 'pulse 2s infinite' : 'none';
    };

    if (isPaused) {
        textEl.textContent = `TOURNOI EN PAUSE${gamesSuffix}`;
        setDotStatus('#f97316');
        return;
    }

    const startDate = startStr ? new Date(startStr) : null;
    const endDate = endStr ? new Date(endStr) : null;

    if (!startDate && !endDate) return;

    function formatDiff(ms) {
        if (ms <= 0) return '00H 00M 00S';
        const totalSec = Math.floor(ms / 1000);
        const hours = Math.floor(totalSec / 3600);
        const minutes = Math.floor((totalSec % 3600) / 60);
        const seconds = totalSec % 60;
        const pad = (n) => String(n).padStart(2, '0');
        return `${pad(hours)}H ${pad(minutes)}M ${pad(seconds)}S`;
    }

    function update() {
        const now = new Date();

        if (startDate && now < startDate) {
            const diff = startDate.getTime() - now.getTime();
            textEl.textContent = `LANCEMENT DANS ${formatDiff(diff)}${gamesSuffix}`;
            setDotStatus('#eab308'); // Jaune / ambre pour événement à venir
        } else if (endDate && now <= endDate) {
            const diff = endDate.getTime() - now.getTime();
            textEl.textContent = `TOURNOI EN COURS · RESTE ${formatDiff(diff)}${gamesSuffix}`;
            setDotStatus('var(--green-status, #10b981)'); // Vert pour en cours
        } else if (endDate && now > endDate) {
            textEl.textContent = `TOURNOI TERMINÉ · CLASSEMENT FINAL VALIDÉ`;
            setDotStatus('#64748b', false, false); // Gris sans pulse pour terminé
        } else {
            textEl.textContent = `TOURNOI EN COURS${gamesSuffix}`;
            setDotStatus('var(--green-status, #10b981)');
        }
    }

    update();
    setInterval(update, 1000);
}

function initGamesCarousel() {
    const track = document.getElementById('carousel-track');
    const dotsContainer = document.getElementById('carousel-dots');
    const counterEl = document.getElementById('carousel-counter');
    const prevBtn = document.getElementById('carousel-prev');
    const nextBtn = document.getElementById('carousel-next');

    if (!track) return;

    const slides = track.querySelectorAll('.carousel-slide');
    const dots = dotsContainer ? dotsContainer.querySelectorAll('.dot') : [];
    const totalSlides = slides.length;

    if (totalSlides === 0) return;

    let currentIndex = 0;

    function updateCarousel() {
        track.style.transform = `translateX(-${currentIndex * 100}%)`;
        if (counterEl) {
            counterEl.textContent = `${String(currentIndex + 1).padStart(2, '0')} / ${String(totalSlides).padStart(2, '0')}`;
        }
        dots.forEach((d, idx) => {
            d.classList.toggle('active', idx === currentIndex);
        });
    }

    function goToSlide(index) {
        if (index < 0) {
            currentIndex = totalSlides - 1;
        } else if (index >= totalSlides) {
            currentIndex = 0;
        } else {
            currentIndex = index;
        }
        updateCarousel();
    }

    dots.forEach((dot, index) => {
        dot.addEventListener('click', () => goToSlide(index));
    });

    if (prevBtn) prevBtn.addEventListener('click', () => goToSlide(currentIndex - 1));
    if (nextBtn) nextBtn.addEventListener('click', () => goToSlide(currentIndex + 1));

    window.addEventListener('keydown', (e) => {
        if (document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA') return;
        if (e.key === 'ArrowLeft') goToSlide(currentIndex - 1);
        if (e.key === 'ArrowRight') goToSlide(currentIndex + 1);
    });

    updateCarousel();
}

