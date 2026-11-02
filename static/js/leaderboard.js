/**
 * Logique du Classement en Direct (leaderboard.js)
 * - Données rendues en SSR côté serveur (Askama)
 * - Actualisation temps réel via WebSocket (LEADERBOARD_UPDATE)
 * - Mode projection plein écran (F11)
 */

import { wsClient } from './ws.js';

document.addEventListener('DOMContentLoaded', () => {
    initWebSocket();
});

function initWebSocket() {
    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', () => {
        window.location.reload();
    });
}
