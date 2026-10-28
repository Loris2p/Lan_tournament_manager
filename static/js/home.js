/**
 * Logique de la page d'accueil (index.html)
 * - Récupération dynamique des jeux configurés via /api/games
 * - Carrousel fluide avec support des boutons, dots et touches clavier
 */

document.addEventListener('DOMContentLoaded', () => {
    initGamesCarousel();
    initFullscreenButton();
});

function initFullscreenButton() {
    const fsBtn = document.getElementById('fullscreen-btn');
    if (!fsBtn) return;
    fsBtn.addEventListener('click', () => {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch((err) => {
                console.warn('Mode plein écran refusé :', err);
            });
        } else {
            document.exitFullscreen().catch(() => {});
        }
    });
}

async function initGamesCarousel() {
    const track = document.getElementById('carousel-track');
    const dotsContainer = document.getElementById('carousel-dots');
    const counterEl = document.getElementById('carousel-counter');
    const prevBtn = document.getElementById('carousel-prev');
    const nextBtn = document.getElementById('carousel-next');

    if (!track) return;

    let games = [];
    let currentIndex = 0;

    try {
        const response = await fetch('/api/games');
        if (response.ok) {
            games = await response.json();
        }
    } catch (err) {
        console.warn('Impossible de charger les jeux depuis /api/games :', err);
    }

    // Si aucun jeu en BDD, on conserve la slide d'exemple
    if (!games || games.length === 0) {
        if (counterEl) counterEl.textContent = '01 / 01';
        return;
    }

    // Construction dynamique des slides
    track.innerHTML = '';
    if (dotsContainer) dotsContainer.innerHTML = '';

    games.forEach((game, index) => {
        const slide = document.createElement('article');
        slide.className = 'carousel-slide';

        const formatLabel = game.format === 'duel' ? 'DUEL 1V1' : 'BRAWL // MULTI-ÉQUIPES';
        const placementText = Array.isArray(game.placement_points) && game.placement_points.length > 0
            ? game.placement_points.map((pt, i) => `${i + 1}e: ${pt}pts`).slice(0, 4).join(', ') + (game.placement_points.length > 4 ? '...' : '')
            : 'Non configuré';

        const secondaryText = game.secondary_stat_label
            ? `${game.secondary_stat_label} (${game.points_per_secondary_stat} pt/unité, max ${game.max_secondary_stat})`
            : 'Aucune statistique secondaire';

        const rulesText = game.rules && game.rules.trim().length > 0
            ? game.rules
            : 'Consignes de match standard. Respecter le fair-play et les directives des arbitres.';

        const mediaMarkup = game.cover_image && game.cover_image.trim().length > 0
            ? `<img src="${encodeURI(game.cover_image)}" alt="${escapeHtml(game.name)}" loading="lazy">`
            : `<div class="slide-image-placeholder"><span class="media-tag">${escapeHtml(game.name.toUpperCase())}</span></div>`;

        slide.innerHTML = `
            <div class="slide-grid">
                <div class="slide-media">
                    ${mediaMarkup}
                </div>
                <div class="slide-content">
                    <div>
                        <div class="slide-header">
                            <span class="badge-format">${formatLabel}</span>
                            <span class="badge-code">DISCIPLINE #${String(index + 1).padStart(2, '0')}</span>
                        </div>
                        <h3 class="slide-title">${escapeHtml(game.name)}</h3>
                        <p class="slide-desc">${escapeHtml(rulesText)}</p>
                    </div>
                    <div class="slide-specs">
                        <div class="slide-spec-row">
                            <span class="spec-k">BARÈME PLACEMENT :</span>
                            <span class="spec-v">${escapeHtml(placementText)}</span>
                        </div>
                        <div class="slide-spec-row">
                            <span class="spec-k">POINTS SECONDAIRES :</span>
                            <span class="spec-v">${escapeHtml(secondaryText)}</span>
                        </div>
                        <div class="slide-spec-row">
                            <span class="spec-k">MAX PLACEMENT :</span>
                            <span class="spec-v">${game.max_placement} équipes</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
        track.appendChild(slide);

        // Dot correspondant
        if (dotsContainer) {
            const dot = document.createElement('span');
            dot.className = `dot ${index === 0 ? 'active' : ''}`;
            dot.addEventListener('click', () => goToSlide(index));
            dotsContainer.appendChild(dot);
        }
    });

    function updateCarousel() {
        track.style.transform = `translateX(-${currentIndex * 100}%)`;
        if (counterEl) {
            counterEl.textContent = `${String(currentIndex + 1).padStart(2, '0')} / ${String(games.length).padStart(2, '0')}`;
        }
        if (dotsContainer) {
            const dots = dotsContainer.querySelectorAll('.dot');
            dots.forEach((d, idx) => {
                d.classList.toggle('active', idx === currentIndex);
            });
        }
    }

    function goToSlide(index) {
        if (index < 0) {
            currentIndex = games.length - 1;
        } else if (index >= games.length) {
            currentIndex = 0;
        } else {
            currentIndex = index;
        }
        updateCarousel();
    }

    if (prevBtn) prevBtn.addEventListener('click', () => goToSlide(currentIndex - 1));
    if (nextBtn) nextBtn.addEventListener('click', () => goToSlide(currentIndex + 1));

    // Navigation clavier flèches gauche / droite sur la section
    window.addEventListener('keydown', (e) => {
        if (document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA') return;
        if (e.key === 'ArrowLeft') goToSlide(currentIndex - 1);
        if (e.key === 'ArrowRight') goToSlide(currentIndex + 1);
    });

    updateCarousel();
}

function escapeHtml(str) {
    if (!str) return '';
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
