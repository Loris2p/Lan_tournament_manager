import { api } from './api.js';
import { wsClient } from './ws.js';
import { showToast, openModal, closeModal, initModalListeners, escapeHtml } from './ui.js';

let cachedGames = null;
let cachedTeams = null;
let cachedMatches = null;

document.addEventListener('DOMContentLoaded', () => {
    initModalListeners();
    initTabs();
    initWebSocket();
    initSubmissions();
    initGames();
    initMatches();
    initTeams();
    initBulkScoresForm();
    initResetZone();
});

function initWebSocket() {
    wsClient.connect();

    wsClient.on('NEW_SUBMISSION', () => {
        showToast("Nouveau score en attente d'arbitrage !", "info");
        const badge = document.getElementById('pending-badge');
        if (badge) {
            badge.textContent = (parseInt(badge.textContent, 10) || 0) + 1;
        }
    });

    wsClient.on('LEADERBOARD_UPDATE', () => {
        showToast("Le classement a été mis à jour !", "info");
    });
}

function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabPanes = document.querySelectorAll('.tab-pane');

    const activateTab = (targetTab) => {
        const btn = document.querySelector(`.tab-btn[data-tab="${targetTab}"]`);
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

function initSubmissions() {
    const listEl = document.getElementById('submissions-list');
    if (listEl) {
        listEl.querySelectorAll('.sub-media').forEach(media => {
            media.addEventListener('click', () => {
                const url = media.getAttribute('data-zoom-url');
                const zoomImg = document.getElementById('zoom-image-src');
                if (zoomImg && url) {
                    zoomImg.src = url;
                    openModal('modal-image-zoom');
                }
            });
        });
    }

    document.getElementById('refresh-submissions-btn')?.addEventListener('click', () => {
        window.location.reload();
    });
}

function initGames() {
    const tbody = document.getElementById('admin-games-tbody');
    if (!tbody) return;

    tbody.querySelectorAll('.btn-del-game').forEach(btn => {
        btn.addEventListener('click', async () => {
            const id = btn.getAttribute('data-id');
            if (confirm("Supprimer ce jeu du catalogue ?")) {
                try {
                    await api.delete(`/api/admin/games/${id}`);
                    showToast("Jeu supprimé.", "info");
                    window.location.reload();
                } catch (err) {
                    showToast(err.message, "error");
                }
            }
        });
    });
}

function initMatches() {
    const container = document.getElementById('matches-admin-list');
    if (!container) return;

    container.querySelectorAll('.btn-edit-instructions').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            const oldInstructions = btn.getAttribute('data-instructions');
            const newInstructions = prompt("Nouvelles consignes / instructions de lobby :", oldInstructions);
            if (newInstructions !== null) {
                const form = document.createElement('form');
                form.method = 'POST';
                form.action = `/admin/matches/${id}/instructions`;
                const input = document.createElement('input');
                input.type = 'hidden';
                input.name = 'instructions';
                input.value = newInstructions;
                form.appendChild(input);
                document.body.appendChild(form);
                form.submit();
            }
        });
    });

    container.querySelectorAll('.btn-bulk-scores').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            openBulkScoresModal(id);
        });
    });
}

async function openBulkScoresModal(matchId) {
    try {
        if (!cachedMatches) cachedMatches = await api.get('/api/admin/matches');
        if (!cachedGames) cachedGames = await api.get('/api/games');
        if (!cachedTeams) cachedTeams = await api.get('/api/leaderboard');
    } catch (err) {
        showToast("Erreur lors de la récupération des données : " + err.message, "error");
        return;
    }

    const match = cachedMatches.find(m => m.id === matchId);
    if (!match) {
        showToast("Match introuvable.", "error");
        return;
    }

    document.getElementById('bulk-match-id').value = match.id;
    document.getElementById('bulk-match-title').textContent = `Saisie Groupée : ${match.game_name || match.title || 'Match'}`;

    const tbody = document.getElementById('bulk-scores-tbody');
    const secHeader = document.getElementById('bulk-sec-header');

    const game = cachedGames.find(g => g.id === match.game_id);
    const hasSec = Boolean(game?.secondary_stat_label);
    if (secHeader) {
        secHeader.textContent = hasSec ? game.secondary_stat_label.toUpperCase() : 'STAT SECONDAIRE (N/A)';
    }

    const participatingTeams = cachedTeams.filter(t => (match.assigned_teams || []).includes(t.id));

    if (participatingTeams.length === 0) {
        tbody.innerHTML = `<tr><td colspan="3" class="cell-loading">Aucune équipe assignée à ce match.</td></tr>`;
    } else {
        tbody.innerHTML = participatingTeams.map((t, idx) => `
            <tr data-team-id="${t.id}">
                <td><strong>${escapeHtml(t.name)}</strong></td>
                <td>
                    <input type="number" class="bulk-input-placement" min="1" value="${idx + 1}" required style="width: 80px;">
                </td>
                <td>
                    <input type="number" class="bulk-input-secondary" min="0" value="0" ${hasSec ? '' : 'disabled'} style="width: 80px;">
                </td>
            </tr>
        `).join('');
    }

    openModal('modal-bulk-scores');
}

function initBulkScoresForm() {
    const form = document.getElementById('bulk-scores-form');
    if (!form) return;

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const matchId = document.getElementById('bulk-match-id').value;
        const rows = document.querySelectorAll('#bulk-scores-tbody tr');

        const scores = [];
        rows.forEach(row => {
            const team_id = row.getAttribute('data-team-id');
            const placement = parseInt(row.querySelector('.bulk-input-placement')?.value, 10);
            const secondary_stat = parseInt(row.querySelector('.bulk-input-secondary')?.value, 10) || 0;
            if (team_id && !isNaN(placement)) {
                scores.push({
                    team_id,
                    score: { placement, secondary_stat }
                });
            }
        });

        try {
            await api.post(`/api/admin/matches/${matchId}/scores`, {
                results: scores,
                close_match: true
            });
            showToast("Scores enregistrés et validés !", "success");
            closeModal('modal-bulk-scores');
            window.location.reload();
        } catch (err) {
            showToast(err.message, "error");
        }
    });
}

function initTeams() {
    const tbody = document.getElementById('admin-teams-tbody');
    if (!tbody) return;

    tbody.querySelectorAll('.btn-adjust-pts').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.getAttribute('data-id');
            const name = btn.getAttribute('data-name');
            const val = prompt(`Ajuster les points pour "${name}" (ex: 10 pour bonus, -5 pour pénalité) :`);
            if (val !== null) {
                const adjustment = parseInt(val, 10);
                if (!isNaN(adjustment)) {
                    const form = document.createElement('form');
                    form.method = 'POST';
                    form.action = `/admin/teams/${id}/points`;
                    const input = document.createElement('input');
                    input.type = 'hidden';
                    input.name = 'adjustment';
                    input.value = adjustment;
                    form.appendChild(input);
                    document.body.appendChild(form);
                    form.submit();
                }
            }
        });
    });
}

function initResetZone() {
    document.querySelectorAll('[data-reset-scope]').forEach(btn => {
        btn.addEventListener('click', () => {
            const rawScope = btn.getAttribute('data-reset-scope');
            const titleEl = document.getElementById('reset-scope-title');
            const descEl = document.getElementById('reset-scope-desc');
            const inputEl = document.getElementById('reset-confirm-input');
            const scopeInput = document.getElementById('reset-scope-input');
            const confirmBtn = document.getElementById('confirm-reset-execute-btn');

            if (!titleEl || !descEl || !inputEl || !confirmBtn) return;

            inputEl.value = '';
            confirmBtn.disabled = true;

            let scopeVal = 'scores_only';
            if (rawScope === 'ScoresOnly') {
                scopeVal = 'scores_only';
                titleEl.textContent = "Remise à zéro des Scores";
                descEl.textContent = "Attention : tous les scores et soumissions actuels seront supprimés. Les compteurs de points repartiront à zéro.";
            } else if (rawScope === 'UsersAndTeams') {
                scopeVal = 'users_and_teams';
                titleEl.textContent = "Purge des Équipes & Joueurs";
                descEl.textContent = "Attention : toutes les équipes et leurs capitaines seront définitivement effacés.";
            } else {
                scopeVal = 'full_tournament';
                titleEl.textContent = "Reset Total du Tournoi";
                descEl.textContent = "DANGER EXTRÊME : purge intégrale de la base de données. Seul votre compte administrateur sera préservé.";
            }

            if (scopeInput) scopeInput.value = scopeVal;
            openModal('modal-confirm-reset');
        });
    });

    const resetInput = document.getElementById('reset-confirm-input');
    const executeResetBtn = document.getElementById('confirm-reset-execute-btn');

    resetInput?.addEventListener('input', () => {
        if (executeResetBtn) {
            executeResetBtn.disabled = resetInput.value.trim().toUpperCase() !== 'RESET';
        }
    });
}
