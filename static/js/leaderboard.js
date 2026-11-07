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

    const activateTab = (targetTab, updateHistory = true) => {
        const btn = document.querySelector(`#leaderboard-tabs-nav .tab-btn[data-tab="${targetTab}"]`);
        const pane = document.getElementById(`tab-${targetTab}`);
        if (btn && pane) {
            tabButtons.forEach(b => b.classList.remove('active'));
            tabPanes.forEach(p => p.classList.remove('active'));
            btn.classList.add('active');
            pane.classList.add('active');

            if (updateHistory) {
                const url = new URL(window.location);
                url.searchParams.set('tab', targetTab);
                window.history.replaceState({}, '', url);
            }
        }
    };

    const currentTab = new URLSearchParams(window.location.search).get('tab');
    if (currentTab) {
        activateTab(currentTab, false);
    }

    tabButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.getAttribute('data-tab');
            activateTab(targetTab, true);
        });
    });

    initAutoKiosk(activateTab);
}

function initAutoKiosk(activateTab) {
    const kioskBtn = document.getElementById('auto-kiosk-btn');
    if (!kioskBtn) return;

    let kioskInterval = null;
    const DURATION = 15000; // 15 secondes par rotation

    const getTabs = () => Array.from(document.querySelectorAll('#leaderboard-tabs-nav .tab-btn'));

    const rotateNext = () => {
        const tabs = getTabs();
        if (tabs.length <= 1) return;
        const activeIdx = tabs.findIndex(t => t.classList.contains('active'));
        const nextIdx = (activeIdx + 1) % tabs.length;
        const nextTarget = tabs[nextIdx].getAttribute('data-tab');
        activateTab(nextTarget, true);
    };

    kioskBtn.addEventListener('click', () => {
        if (kioskInterval) {
            clearInterval(kioskInterval);
            kioskInterval = null;
            kioskBtn.textContent = '⏱️ Défilement Auto : OFF';
            kioskBtn.classList.remove('btn-primary');
            kioskBtn.classList.add('btn-outline');
        } else {
            rotateNext();
            kioskInterval = setInterval(rotateNext, DURATION);
            kioskBtn.textContent = '⏱️ Défilement Auto : ON (15s)';
            kioskBtn.classList.remove('btn-outline');
            kioskBtn.classList.add('btn-primary');
        }
    });
}

function initWebSocket() {
    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', async () => {
        await refreshLeaderboardSilently();
    });
}

async function refreshLeaderboardSilently() {
    try {
        const res = await fetch(window.location.href);
        if (!res.ok) {
            window.location.reload();
            return;
        }
        const html = await res.text();
        const doc = new DOMParser().parseFromString(html, 'text/html');

        // Éléments du classement général à mettre à jour
        const selectors = [
            '#standings-count',
            '#podium-container',
            '#leaderboard-tbody',
            '#live-matches-list'
        ];

        selectors.forEach(sel => {
            const currentEl = document.querySelector(sel);
            const newEl = doc.querySelector(sel);
            if (currentEl && newEl) {
                currentEl.innerHTML = newEl.innerHTML;
            }
        });

        // Mise à jour de chaque onglet par épreuve
        document.querySelectorAll('.tab-pane[id^="tab-game-"]').forEach(pane => {
            const newPane = doc.getElementById(pane.id);
            if (newPane) {
                pane.innerHTML = newPane.innerHTML;
            }
        });

        // Flash visuel subtil sur le voyant direct pour signaler la synchronisation
        const dot = document.querySelector('.status-dot');
        if (dot) {
            dot.style.transform = 'scale(1.5)';
            setTimeout(() => { dot.style.transform = ''; }, 500);
        }
    } catch (e) {
        console.warn("Échec du rafraîchissement silencieux, fallback rechargement:", e);
        window.location.reload();
    }
}
