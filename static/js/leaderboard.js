/**
 * Logique du Classement en Direct (leaderboard.js)
 * - Données rendues en SSR côté serveur (Askama)
 * - Actualisation temps réel via WebSocket (LEADERBOARD_UPDATE)
 * - Mode projection plein écran (F11)
 */

import { wsClient } from './ws.js';

document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initWebSocket();
});

function initTabs() {
    const tabButtons = document.querySelectorAll('#leaderboard-tabs-nav .tab-btn');
    const tabPanes = document.querySelectorAll('.tab-pane');

    const activateTab = (targetTab) => {
        const btn = document.querySelector(`#leaderboard-tabs-nav .tab-btn[data-tab="${targetTab}"]`);
        const pane = document.getElementById(`tab-${targetTab}`);
        if (btn && pane) {
            tabButtons.forEach(b => b.classList.remove('active'));
            tabPanes.forEach(p => p.classList.remove('active'));
            btn.classList.add('active');
            pane.classList.add('active');
        }
    };

    const currentTab = new URLSearchParams(window.location.search).get('tab');
    if (currentTab) {
        activateTab(currentTab);
    }

    tabButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.getAttribute('data-tab');
            activateTab(targetTab);
            const url = new URL(window.location);
            url.searchParams.set('tab', targetTab);
            window.history.replaceState({}, '', url);
        });
    });
}

function initWebSocket() {
    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', () => {
        window.location.reload();
    });
}
