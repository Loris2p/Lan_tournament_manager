/**
 * Logique de la page d'accueil (index.html)
 * - Données rendues en SSR (Askama)
 * - Carrousel fluide avec support des boutons, dots et touches clavier
 */

import { wsClient } from './ws.js';
import { openModal, initModalListeners } from './ui.js';

document.addEventListener('DOMContentLoaded', () => {
    wsClient.connect();
    initGamesCarousel();
    initFullscreenButton();
    initLegalModals();
});

function initLegalModals() {
    initModalListeners();

    const btnLegal = document.getElementById('btn-legal');
    const btnPrivacy = document.getElementById('btn-privacy');
    const titleEl = document.getElementById('legal-modal-title');
    const bodyEl = document.getElementById('legal-modal-body');

    if (btnLegal) {
        btnLegal.addEventListener('click', () => {
            if (titleEl) titleEl.textContent = 'MENTIONS LÉGALES';
            if (bodyEl) {
                bodyEl.innerHTML = `
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Édition & Organisation</h4>
                    <p style="margin-bottom: 1rem;">
                        Ce service est déployé localement dans le cadre de l'événement associatif / LAN Tournament Manager. 
                        Il est destiné exclusivement à la gestion interne des scores, des arbitres et des équipes participantes.
                    </p>
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Hébergement</h4>
                    <p style="margin-bottom: 1rem;">
                        Serveur autonome exécuté sur site (LAN locale). Aucune donnée n'est hébergée sur des serveurs tiers ni transmise à l'extérieur.
                    </p>
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Propriété Intellectuelle</h4>
                    <p>
                        Les titres, logos, visuels et marques des jeux mentionnés demeurent la propriété exclusive de leurs éditeurs et ayants droit respectifs.
                    </p>
                `;
            }
            openModal('modal-legal');
        });
    }

    if (btnPrivacy) {
        btnPrivacy.addEventListener('click', () => {
            if (titleEl) titleEl.textContent = 'CONFIDENTIALITÉ & DONNÉES PERSONNELLES';
            if (bodyEl) {
                bodyEl.innerHTML = `
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Cookies Techniques</h4>
                    <p style="margin-bottom: 1rem;">
                        Ce site utilise exclusivement un cookie technique sécurisé (<code>lan_session</code>) strictement indispensable à l'authentification des capitaines et administrateurs. Aucun cookie tiers, traceur publicitaire ou outil de profilage n'est employé.
                    </p>
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Données collectées & RGPD</h4>
                    <p style="margin-bottom: 1rem;">
                        Les seules informations enregistrées sont les pseudonymes d'équipe, les scores déclarés et les captures d'écran de preuve de match transmises par les capitaines.
                    </p>
                    <h4 style="color: var(--text-primary); margin-bottom: 0.5rem;">Conservation</h4>
                    <p>
                        Les données sont conservées pour la seule durée du tournoi et peuvent être purgées à tout moment via la console d'administration.
                    </p>
                `;
            }
            openModal('modal-legal');
        });
    }
}

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

function escapeHtml(str) {
    if (!str) return '';
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
