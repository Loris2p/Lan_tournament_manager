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
    initGamePresets();
    initMatches();
    initTeams();
    initTournamentSettings();
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

function initTournamentSettings() {
    const form = document.getElementById('settings-points-form');
    const input = document.getElementById('tournament-placement-points');
    const defaultBtn = document.getElementById('btn-default-points');
    const statusMsg = document.getElementById('settings-status-msg');

    if (!form || !input) return;

    // Charger les paramètres actuels
    api.get('/api/admin/settings')
        .then(settings => {
            if (settings && Array.isArray(settings.global_placement_points)) {
                input.value = settings.global_placement_points.join(', ');
            }
        })
        .catch(err => {
            console.warn("Impossible de charger les paramètres de tournoi:", err);
        });

    // Bouton de réinitialisation vers le barème par défaut (10 équipes)
    defaultBtn?.addEventListener('click', () => {
        input.value = '25, 18, 15, 12, 10, 8, 6, 4, 2, 1';
        showToast("Barème standard par défaut (10 équipes) chargé.", "info");
    });

    // Envoi du formulaire de barème
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const rawValues = input.value
            .split(',')
            .map(s => parseInt(s.trim(), 10))
            .filter(n => !isNaN(n));

        if (rawValues.length === 0) {
            showToast("Veuillez saisir au moins une valeur de point.", "error");
            return;
        }

        try {
            await api.post('/api/admin/settings', { global_placement_points: rawValues });
            showToast("Barème général du tournoi enregistré avec succès !", "success");
            if (statusMsg) {
                statusMsg.textContent = "✓ Enregistré";
                statusMsg.style.color = "var(--green, #22c55e)";
                setTimeout(() => { statusMsg.textContent = ""; }, 3000);
            }
        } catch (err) {
            showToast(err.message, "error");
        }
    });
}

function initGamePresets() {
    const presets = {
        apex: {
            name: "Apex Legends",
            format: "brawl",
            cover: "",
            placement: "12, 9, 7, 5, 4, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0",
            defPoints: 0,
            maxPlacement: 20,
            secLabel: "Frags (Éliminations)",
            secPts: 1,
            maxSec: 60,
            rules: "Salon personnalisé Trios. Barème ALGS (Placement + 1 pt/frag). Cumul sur toute la session de 4 heures."
        },
        lol: {
            name: "League of Legends",
            format: "duel",
            cover: "",
            placement: "3, 1",
            defPoints: 0,
            maxPlacement: 2,
            secLabel: "",
            secPts: 0,
            maxSec: 0,
            rules: "ARAM du chaos. Duels en 1 manche gagnante (Victoire: 3 pts, Défaite: 1 pt, Forfait: 0 pt). Championnat puis finale pour les 2 meilleures équipes."
        },
        fallguys: {
            name: "Fall Guys",
            format: "brawl",
            cover: "",
            placement: "3, 0",
            defPoints: 0,
            maxPlacement: 2,
            secLabel: "Manches franchies (cumul 3 joueurs)",
            secPts: 1,
            maxSec: 15,
            rules: "Salon personnalisé Solo (3 joueurs/équipe). +1 pt par manche franchie par joueur. Prime de victoire de 3 pts en cas de victoire du show (place 1)."
        },
        geoguessr: {
            name: "GeoGuessr",
            format: "brawl",
            cover: "",
            placement: "0",
            defPoints: 0,
            maxPlacement: 1,
            secLabel: "Score total en points",
            secPts: 1,
            maxSec: 75000,
            rules: "Challenge d'équipe coopératif (3 joueurs). 15 rounds sur chaque carte (World, A Community World, France, F1, etc.). Mode No Move 1m15/round. Cumul des points bruts."
        },
        ssbu: {
            name: "Super Smash Bros Ultimate",
            format: "duel",
            cover: "",
            placement: "3, 1",
            defPoints: 0,
            maxPlacement: 2,
            secLabel: "",
            secPts: 0,
            maxSec: 0,
            rules: "Smash en bande 3v3 (3 vies, sans objets/smash final, maps CB autorisées). Bans de 3 persos/équipe. Victoire: 3 pts, Défaite: 1 pt. Finale BO3 pour le top 2."
        },
        mk8: {
            name: "Mario Kart 8 Deluxe",
            format: "brawl",
            cover: "",
            placement: "0",
            defPoints: 0,
            maxPlacement: 1,
            secLabel: "Points de course cumulés (3 joueurs)",
            secPts: 1,
            maxSec: 200,
            rules: "Lobbies LAN 12 joueurs (pseudo-solo, 3 joueurs/équipe). Sessions de 4 courses 150cc sans IA. Cumul des points de course in-game des 3 joueurs."
        }
    };

    document.querySelectorAll('.btn-game-preset').forEach(btn => {
        btn.addEventListener('click', () => {
            const presetKey = btn.getAttribute('data-preset');
            const data = presets[presetKey];
            if (!data) return;

            const nameEl = document.getElementById('game-name');
            const formatEl = document.getElementById('game-format');
            const coverEl = document.getElementById('game-cover');
            const placementEl = document.getElementById('game-placement-points');
            const defPointsEl = document.getElementById('game-default-points');
            const maxPlacementEl = document.getElementById('game-max-placement');
            const secLabelEl = document.getElementById('game-sec-label');
            const secPtsEl = document.getElementById('game-sec-pts');
            const maxSecEl = document.getElementById('game-max-sec');
            const rulesEl = document.getElementById('game-rules');

            if (nameEl) nameEl.value = data.name;
            if (formatEl) formatEl.value = data.format;
            if (coverEl) coverEl.value = data.cover;
            if (placementEl) placementEl.value = data.placement;
            if (defPointsEl) defPointsEl.value = data.defPoints;
            if (maxPlacementEl) maxPlacementEl.value = data.maxPlacement;
            if (secLabelEl) secLabelEl.value = data.secLabel;
            if (secPtsEl) secPtsEl.value = data.secPts;
            if (maxSecEl) maxSecEl.value = data.maxSec;
            if (rulesEl) rulesEl.value = data.rules;

            showToast(`Préréglage "${data.name}" chargé dans le formulaire !`, "info");
        });
    });
}
