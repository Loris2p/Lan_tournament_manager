/**
 * Logique du Classement en Direct (leaderboard.js)
 * - Données rendues en SSR côté serveur (Askama)
 * - Actualisation temps réel via WebSocket (LEADERBOARD_UPDATE)
 * - Mode projection plein écran (F11)
 */

import { wsClient } from './ws.js';
import { checkAuth } from './auth-guard.js';

document.addEventListener('DOMContentLoaded', () => {
    checkAuth(null);
    initFullscreen();
    initWebSocket();
});

function initWebSocket() {
    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', () => {
        window.location.reload();
    });
}

function initFullscreen() {
    const fsBtn = document.getElementById('fullscreen-btn');
    if (!fsBtn) return;

    fsBtn.addEventListener('click', () => {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch(() => {});
        } else {
            document.exitFullscreen().catch(() => {});
        }
    });

    document.addEventListener('fullscreenchange', () => {
        if (document.fullscreenElement) {
            fsBtn.textContent = 'Quitter Plein Écran [Échap]';
            document.body.classList.add('fullscreen-mode');
        } else {
            fsBtn.textContent = 'Mode Écran Géant [F11]';
            document.body.classList.remove('fullscreen-mode');
        }
    });
}
