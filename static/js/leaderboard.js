/**
 * Logique du Classement en Direct (leaderboard.js)
 * Récupère le classement (/api/leaderboard), les matchs en cours (/api/matches),
 * gère la diffusion temps réel via WebSocket (LEADERBOARD_UPDATE)
 * et le mode projection plein écran.
 */

import { api } from './api.js';
import { wsClient } from './ws.js';
import { checkAuth } from './auth-guard.js';
import { escapeHtml } from './ui.js';

document.addEventListener('DOMContentLoaded', () => {
    checkAuth(null); // Met à jour le header si l'utilisateur est connecté
    initLeaderboard();
    initFullscreen();
    initWebSocket();
});

async function initLeaderboard() {
    await Promise.all([
        fetchLeaderboard(),
        fetchLiveMatches()
    ]);
}

function initWebSocket() {
    wsClient.connect();
    // Rafraîchissement automatique lors de la validation d'un score
    wsClient.on('LEADERBOARD_UPDATE', () => {
        fetchLeaderboard();
        fetchLiveMatches();
    });
}

async function fetchLeaderboard() {
    const tbody = document.getElementById('leaderboard-tbody');
    const countEl = document.getElementById('standings-count');

    try {
        const teams = await api.get('/api/leaderboard');
        // teams = Array of { name: "...", points: 120, ... }

        if (countEl) {
            countEl.textContent = `${teams.length} ÉQUIPE${teams.length > 1 ? 'S' : ''} ENREGISTRÉE${teams.length > 1 ? 'S' : ''}`;
        }

        renderPodium(teams);
        renderTable(teams, tbody);
    } catch (err) {
        if (tbody) {
            tbody.innerHTML = `<tr><td colspan="3" class="cell-loading" style="color: var(--red);">Erreur lors du chargement des scores : ${escapeHtml(err.message)}</td></tr>`;
        }
    }
}

function renderPodium(teams) {
    const first = teams[0] || null;
    const second = teams[1] || null;
    const third = teams[2] || null;

    // 1er Place
    const p1Name = document.getElementById('podium-1-name');
    const p1Points = document.getElementById('podium-1-points');
    if (p1Name && p1Points) {
        p1Name.textContent = first ? first.name : '--';
        p1Points.textContent = first ? `${first.total_points ?? first.points ?? 0} PTS` : '--';
    }

    // 2ème Place
    const p2Name = document.getElementById('podium-2-name');
    const p2Points = document.getElementById('podium-2-points');
    if (p2Name && p2Points) {
        p2Name.textContent = second ? second.name : '--';
        p2Points.textContent = second ? `${second.total_points ?? second.points ?? 0} PTS` : '--';
    }

    // 3ème Place
    const p3Name = document.getElementById('podium-3-name');
    const p3Points = document.getElementById('podium-3-points');
    if (p3Name && p3Points) {
        p3Name.textContent = third ? third.name : '--';
        p3Points.textContent = third ? `${third.total_points ?? third.points ?? 0} PTS` : '--';
    }
}

function renderTable(teams, tbody) {
    if (!tbody) return;

    if (!teams || teams.length === 0) {
        tbody.innerHTML = `<tr><td colspan="3" class="cell-loading">Aucune équipe inscrite pour le moment.</td></tr>`;
        return;
    }

    // Affichage des équipes à partir du rang 4 (ou tableau complet si moins de 4 équipes)
    const tableTeams = teams.length > 3 ? teams.slice(3) : teams;
    const startRank = teams.length > 3 ? 4 : 1;

    tbody.innerHTML = tableTeams.map((team, idx) => {
        const rank = startRank + idx;
        const rankFormatted = String(rank).padStart(2, '0');
        return `
            <tr>
                <td class="col-rank">${rankFormatted}</td>
                <td class="col-team">${escapeHtml(team.name)}</td>
                <td class="col-points">${team.total_points ?? team.points ?? 0} PTS</td>
            </tr>
        `;
    }).join('');
}

async function fetchLiveMatches() {
    const listEl = document.getElementById('live-matches-list');
    if (!listEl) return;

    try {
        const matches = await api.get('/api/matches');
        // Matchs ouverts (is_open == true)
        if (!matches || matches.length === 0) {
            listEl.innerHTML = `<div class="empty-state">Aucun match ouvert aux soumissions pour le moment.</div>`;
            return;
        }

        listEl.innerHTML = matches.map(match => {
            const instructions = match.instructions && match.instructions.trim().length > 0
                ? match.instructions
                : 'En attente des consignes d\'arbitrage';

            return `
                <div class="match-live-card">
                    <div class="match-live-header">
                        <span class="match-game-title">${escapeHtml(match.game_name || 'Match Officiel')}</span>
                        <span class="match-live-status">EN COURS</span>
                    </div>
                    <div class="match-instructions-box">
                        <strong>LOBBY / CONSIGNES :</strong> ${escapeHtml(instructions)}
                    </div>
                </div>
            `;
        }).join('');
    } catch (_) {
        // En cas d'erreur de récupération des matchs, conserver l'état
    }
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
