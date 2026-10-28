/**
 * Centre de Contrôle Administrateur (admin.js)
 * Gestion des 6 onglets :
 * 1. Arbitrage captures (zoom, approbation, rejet, alerte WebSocket NEW_SUBMISSION)
 * 2. Catalogue de jeux (barèmes dynamiques Duel/Brawl, limites anti-triche)
 * 3. Matchs & Saisie groupée Brawl
 * 4. Équipes, mots de passe capitaines & ajustements manuels de points
 * 5. Gestion des comptes administrateurs
 * 6. Zone de danger (Reset selon les 3 scopes)
 */

import { api } from './api.js';
import { checkAuth, logout } from './auth-guard.js';
import { wsClient } from './ws.js';
import { showToast, openModal, closeModal, initModalListeners, escapeHtml } from './ui.js';

let currentUser = null;
let allGames = [];
let allTeams = [];
let allMatches = [];
let pendingSubmissions = [];
let activeResetScope = null;

document.addEventListener('DOMContentLoaded', async () => {
    currentUser = await checkAuth('admin');
    if (!currentUser) return;

    initModalListeners();
    initAdminHeader();
    initTabs();
    initWebSocket();

    // Chargement initial des données
    await Promise.all([
        loadSubmissions(),
        loadGames(),
        loadTeams(),
        loadMatches(),
        loadAdmins()
    ]);

    initForms();

    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) logoutBtn.addEventListener('click', logout);
});

function initAdminHeader() {
    const badge = document.getElementById('admin-user-badge');
    if (badge) badge.textContent = `ADMIN : ${currentUser.username.toUpperCase()}`;
}

function initWebSocket() {
    wsClient.connect();

    wsClient.on('NEW_SUBMISSION', () => {
        showToast("Nouveau score en attente d'arbitrage !", "info");
        loadSubmissions();
    });

    wsClient.on('LEADERBOARD_UPDATE', () => {
        loadTeams();
    });
}

function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabPanes = document.querySelectorAll('.tab-pane');

    tabButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.getAttribute('data-tab');

            tabButtons.forEach(b => b.classList.remove('active'));
            tabPanes.forEach(p => p.classList.remove('active'));

            btn.classList.add('active');
            const targetPane = document.getElementById(`tab-${targetTab}`);
            if (targetPane) targetPane.classList.add('active');
        });
    });
}

/* ==========================================================================
   ONGLET 1 : ARBITRAGE DES SOUMISSIONS
   ========================================================================== */

async function loadSubmissions() {
    const listEl = document.getElementById('submissions-list');
    const badge = document.getElementById('pending-badge');

    try {
        pendingSubmissions = await api.get('/api/admin/submissions');
        badge.textContent = pendingSubmissions.length;

        if (!pendingSubmissions || pendingSubmissions.length === 0) {
            listEl.innerHTML = `<div class="empty-state">Aucune soumission en attente. Toutes les captures ont été arbitrées !</div>`;
            return;
        }

        listEl.innerHTML = pendingSubmissions.map(sub => {
            const placement = sub.score ? sub.score.placement : '?';
            const secStat = sub.score ? sub.score.secondary_stat : 0;
            const secondaryText = secStat > 0 ? `+ ${secStat} bonus` : '';
            const screenshotUrl = sub.proof_base64 || '';

            return `
                <div class="submission-card" data-sub-id="${sub.id}">
                    <div class="sub-media" data-zoom-url="${screenshotUrl}" title="Cliquer pour agrandir la capture">
                        ${screenshotUrl ? `<img src="${screenshotUrl}" alt="Preuve de score" loading="lazy">` : '<div class="slide-image-placeholder">SANS PHOTO</div>'}
                        <span class="sub-zoom-hint">ZOOM [CLIQUEZ]</span>
                    </div>
                    <div class="sub-content">
                        <div class="sub-header">
                            <span class="sub-team-title">${escapeHtml(sub.team_name)}</span>
                            <span class="badge-code">${sub.calculated_points} PTS ATTENDUS</span>
                        </div>
                        <div class="sub-scores-declared">
                            <span class="sub-placement-pill">PLACE : #${placement}</span>
                            <span class="sub-secondary-pill">${secondaryText}</span>
                        </div>
                        <div class="sub-actions">
                            <button type="button" class="btn-primary btn-approve" data-id="${sub.id}">
                                ✓ Valider & Créditer
                            </button>
                            <button type="button" class="btn-outline btn-reject" data-id="${sub.id}">
                                ✕ Rejeter
                            </button>
                        </div>
                    </div>
                </div>
            `;
        }).join('');

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

        listEl.querySelectorAll('.btn-approve').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.getAttribute('data-id');
                btn.disabled = true;
                try {
                    await api.post(`/api/admin/submissions/${id}/approve`);
                    showToast("Score approuvé ! Points crédités au classement.", "success");
                    loadSubmissions();
                    loadTeams();
                } catch (err) {
                    showToast(err.message, "error");
                    btn.disabled = false;
                }
            });
        });

        listEl.querySelectorAll('.btn-reject').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.getAttribute('data-id');
                btn.disabled = true;
                try {
                    await api.post(`/api/admin/submissions/${id}/reject`);
                    showToast("Soumission rejetée.", "info");
                    loadSubmissions();
                } catch (err) {
                    showToast(err.message, "error");
                    btn.disabled = false;
                }
            });
        });

    } catch (err) {
        listEl.innerHTML = `<div class="empty-state" style="color: var(--red);">Erreur lors du chargement des soumissions : ${err.message}</div>`;
    }
}

document.getElementById('refresh-submissions-btn')?.addEventListener('click', loadSubmissions);

/* ==========================================================================
   ONGLET 2 : CATALOGUE DES JEUX
   ========================================================================== */

async function loadGames() {
    const tbody = document.getElementById('admin-games-tbody');
    const selectEl = document.getElementById('match-game-select');

    try {
        allGames = await api.get('/api/games');

        if (!allGames || allGames.length === 0) {
            tbody.innerHTML = `<tr><td colspan="4" class="cell-loading">Aucun jeu configuré. Ajoutez une épreuve ci-contre.</td></tr>`;
        } else {
            tbody.innerHTML = allGames.map(game => `
                <tr>
                    <td><strong>${escapeHtml(game.name)}</strong></td>
                    <td><span class="badge-format">${game.format === 'duel' ? 'DUEL' : 'BRAWL'}</span></td>
                    <td><code>[${(game.placement_points || []).join(', ')}]</code></td>
                    <td>
                        <button type="button" class="btn-text btn-del-game" data-id="${game.id}" style="color: var(--red);">
                            Supprimer
                        </button>
                    </td>
                </tr>
            `).join('');

            tbody.querySelectorAll('.btn-del-game').forEach(btn => {
                btn.addEventListener('click', async () => {
                    const id = btn.getAttribute('data-id');
                    if (confirm("Supprimer ce jeu du catalogue ?")) {
                        try {
                            await api.delete(`/api/admin/games/${id}`);
                            showToast("Jeu supprimé.", "info");
                            loadGames();
                        } catch (err) {
                            showToast(err.message, "error");
                        }
                    }
                });
            });
        }

        if (selectEl) {
            selectEl.innerHTML = `<option value="">Sélectionnez un jeu...</option>` +
                allGames.map(g => `<option value="${g.id}">${escapeHtml(g.name)} (${g.format.toUpperCase()})</option>`).join('');
        }
    } catch (err) {
        tbody.innerHTML = `<tr><td colspan="4" class="cell-loading" style="color: var(--red);">Erreur : ${err.message}</td></tr>`;
    }
}

/* ==========================================================================
   ONGLET 3 : MATCHS, LOBBIES & SAISIE RAPIDE BRAWL
   ========================================================================== */

async function loadMatches() {
    const container = document.getElementById('matches-admin-list');

    try {
        allMatches = await api.get('/api/admin/matches');

        if (!allMatches || allMatches.length === 0) {
            container.innerHTML = `<div class="empty-state">Aucun match créé. Utilisez le formulaire ci-contre pour programmer une épreuve.</div>`;
            return;
        }

        container.innerHTML = allMatches.map(m => {
            const isOpen = m.is_open;
            const teamsCount = (m.assigned_teams || []).length;

            return `
                <div class="match-admin-card ${isOpen ? 'is-open' : 'is-closed'}">
                    <div class="match-admin-header">
                        <div>
                            <span class="badge-format">${m.title || 'MATCH'}</span>
                            <h4 style="display: inline-block; margin-left: 0.5rem;">${escapeHtml(m.game_name || 'Match')}</h4>
                        </div>
                        <span class="pulse-indicator" style="${isOpen ? '' : 'background: rgba(113, 113, 122, 0.2); color: var(--text-muted);'}">
                            ${isOpen ? 'OUVERT AUX SOUMISSIONS' : 'FERMÉ'}
                        </span>
                    </div>

                    <div class="match-admin-body">
                        <p><strong>Instructions :</strong> ${escapeHtml(m.instructions || 'Aucune consigne')}</p>
                        <p><small>${teamsCount} équipe(s) convoquée(s)</small></p>
                    </div>

                    <div class="match-admin-actions">
                        <button type="button" class="btn-outline btn-toggle-match" data-id="${m.id}">
                            ${isOpen ? 'Fermer le Match' : 'Ouvrir aux Joueurs'}
                        </button>
                        <button type="button" class="btn-text btn-edit-instructions" data-id="${m.id}" data-instructions="${escapeHtml(m.instructions || '')}">
                            Modifier Consignes
                        </button>
                        <button type="button" class="btn-primary btn-bulk-scores" data-id="${m.id}">
                            Saisie Groupée des Scores
                        </button>
                    </div>
                </div>
            `;
        }).join('');

        container.querySelectorAll('.btn-toggle-match').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.getAttribute('data-id');
                try {
                    await api.post(`/api/admin/matches/${id}/toggle`);
                    showToast("Statut du match mis à jour.", "info");
                    loadMatches();
                } catch (err) {
                    showToast(err.message, "error");
                }
            });
        });

        container.querySelectorAll('.btn-edit-instructions').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.getAttribute('data-id');
                const oldInstructions = btn.getAttribute('data-instructions');
                const newInstructions = prompt("Nouvelles consignes / instructions de lobby :", oldInstructions);
                if (newInstructions !== null) {
                    try {
                        await api.patch(`/api/admin/matches/${id}/instructions`, { instructions: newInstructions });
                        showToast("Consignes mises à jour en direct.", "success");
                        loadMatches();
                    } catch (err) {
                        showToast(err.message, "error");
                    }
                }
            });
        });

        container.querySelectorAll('.btn-bulk-scores').forEach(btn => {
            btn.addEventListener('click', () => {
                const id = btn.getAttribute('data-id');
                openBulkScoresModal(id);
            });
        });

    } catch (err) {
        container.innerHTML = `<div class="empty-state" style="color: var(--red);">Erreur : ${err.message}</div>`;
    }
}

function openBulkScoresModal(matchId) {
    const match = allMatches.find(m => m.id === matchId);
    if (!match) return;

    document.getElementById('bulk-match-id').value = match.id;
    document.getElementById('bulk-match-title').textContent = `Saisie Groupée : ${match.game_name || match.title || 'Match'}`;

    const tbody = document.getElementById('bulk-scores-tbody');
    const secHeader = document.getElementById('bulk-sec-header');

    const game = allGames.find(g => g.id === match.game_id);
    const hasSec = game && game.secondary_stat_label;
    secHeader.textContent = hasSec ? game.secondary_stat_label.toUpperCase() : 'STAT SECONDAIRE (N/A)';

    const participatingTeams = allTeams.filter(t => (match.assigned_teams || []).includes(t.id));

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

/* ==========================================================================
   ONGLET 4 : ÉQUIPES & CAPITAINES
   ========================================================================== */

async function loadTeams() {
    const tbody = document.getElementById('admin-teams-tbody');
    const selectorContainer = document.getElementById('match-teams-selector');

    try {
        allTeams = await api.get('/api/leaderboard');

        if (!allTeams || allTeams.length === 0) {
            tbody.innerHTML = `<tr><td colspan="4" class="cell-loading">Aucune équipe inscrite.</td></tr>`;
        } else {
            tbody.innerHTML = allTeams.map(team => `
                <tr>
                    <td><strong>${escapeHtml(team.name)}</strong></td>
                    <td class="col-points">${team.points} PTS</td>
                    <td>
                        <button type="button" class="btn-text btn-adjust-pts" data-id="${team.id}" data-name="${escapeHtml(team.name)}">
                            +/- Ajuster
                        </button>
                    </td>
                    <td>
                        <button type="button" class="btn-text btn-del-team" data-id="${team.id}" style="color: var(--red);">
                            Supprimer
                        </button>
                    </td>
                </tr>
            `).join('');

            tbody.querySelectorAll('.btn-adjust-pts').forEach(btn => {
                btn.addEventListener('click', async () => {
                    const id = btn.getAttribute('data-id');
                    const name = btn.getAttribute('data-name');
                    const val = prompt(`Ajuster les points pour "${name}" (ex: 10 pour bonus, -5 pour pénalité) :`);
                    if (val !== null) {
                        const adjustment = parseInt(val, 10);
                        if (!isNaN(adjustment)) {
                            try {
                                await api.patch(`/api/admin/teams/${id}/points`, { adjustment });
                                showToast("Points mis à jour !", "success");
                                loadTeams();
                            } catch (err) {
                                showToast(err.message, "error");
                            }
                        }
                    }
                });
            });

            tbody.querySelectorAll('.btn-del-team').forEach(btn => {
                btn.addEventListener('click', async () => {
                    const id = btn.getAttribute('data-id');
                    if (confirm("Supprimer définitivement cette équipe et son compte capitaine ?")) {
                        try {
                            await api.delete(`/api/admin/teams/${id}`);
                            showToast("Équipe supprimée.", "info");
                            loadTeams();
                        } catch (err) {
                            showToast(err.message, "error");
                        }
                    }
                });
            });
        }

        if (selectorContainer) {
            if (allTeams.length === 0) {
                selectorContainer.innerHTML = `<span class="field-hint">Créez d'abord des équipes dans l'onglet Équipes.</span>`;
            } else {
                selectorContainer.innerHTML = allTeams.map(t => `
                    <label class="team-checkbox-label">
                        <input type="checkbox" name="match-teams" value="${t.id}">
                        <span>${escapeHtml(t.name)}</span>
                    </label>
                `).join('');
            }
        }
    } catch (err) {
        tbody.innerHTML = `<tr><td colspan="4" class="cell-loading" style="color: var(--red);">Erreur : ${err.message}</td></tr>`;
    }
}

/* ==========================================================================
   ONGLET 5 : COMPTES ADMINISTRATEURS
   ========================================================================== */

async function loadAdmins() {
    const tbody = document.getElementById('admin-users-tbody');

    try {
        const admins = await api.get('/api/admin/admins');

        if (!admins || admins.length === 0) {
            tbody.innerHTML = `<tr><td colspan="2" class="cell-loading">Aucun administrateur trouvé.</td></tr>`;
            return;
        }

        tbody.innerHTML = admins.map(adm => {
            const isSelf = adm.username.toLowerCase() === currentUser.username.toLowerCase();
            return `
                <tr>
                    <td><strong>${escapeHtml(adm.username)}</strong> ${isSelf ? '<span class="badge-code">[VOUS]</span>' : ''}</td>
                    <td>
                        ${isSelf ? '<span class="field-hint">Actif</span>' : `
                            <button type="button" class="btn-text btn-del-admin" data-id="${adm.id}" style="color: var(--red);">
                                Révoquer
                            </button>
                        `}
                    </td>
                </tr>
            `;
        }).join('');

        tbody.querySelectorAll('.btn-del-admin').forEach(btn => {
            btn.addEventListener('click', async () => {
                const id = btn.getAttribute('data-id');
                if (confirm("Supprimer ce compte administrateur ?")) {
                    try {
                        await api.delete(`/api/admin/admins/${id}`);
                        showToast("Administrateur révoqué.", "info");
                        loadAdmins();
                    } catch (err) {
                        showToast(err.message, "error");
                    }
                }
            });
        });
    } catch (err) {
        tbody.innerHTML = `<tr><td colspan="2" class="cell-loading" style="color: var(--red);">Erreur : ${err.message}</td></tr>`;
    }
}

/* ==========================================================================
   INITIALISATION DES FORMULAIRES ADMIN
   ========================================================================== */

function initForms() {
    // 1. Création de Jeu
    document.getElementById('create-game-form')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const name = document.getElementById('game-name').value.trim();
        const format = document.getElementById('game-format').value;
        const cover_image = document.getElementById('game-cover').value.trim();
        const placementStr = document.getElementById('game-placement-points').value;
        const defaultPts = parseInt(document.getElementById('game-default-points').value, 10) || 0;
        const maxPlacement = parseInt(document.getElementById('game-max-placement').value, 10) || 2;
        const secLabel = document.getElementById('game-sec-label').value.trim();
        const secPts = parseInt(document.getElementById('game-sec-pts').value, 10) || 0;
        const maxSec = parseInt(document.getElementById('game-max-sec').value, 10) || 0;
        const rules = document.getElementById('game-rules').value.trim();

        const placement_points = placementStr.split(',').map(s => parseInt(s.trim(), 10)).filter(n => !isNaN(n));

        try {
            await api.post('/api/admin/games', {
                name,
                format,
                cover_image: cover_image || "",
                placement_points,
                default_placement_points: defaultPts,
                secondary_stat_label: secLabel.length > 0 ? secLabel : undefined,
                points_per_secondary_stat: secPts,
                max_placement: maxPlacement,
                max_secondary_stat: maxSec,
                rules
            });
            showToast("Jeu ajouté avec succès au catalogue !", "success");
            e.target.reset();
            loadGames();
        } catch (err) {
            showToast(err.message, "error");
        }
    });

    // 2. Création de Match
    document.getElementById('create-match-form')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const game_id = document.getElementById('match-game-select').value;
        const instructions = document.getElementById('match-instructions').value.trim();

        const checkedTeams = Array.from(document.querySelectorAll('input[name="match-teams"]:checked'))
            .map(cb => cb.value);

        if (!game_id) {
            showToast("Veuillez sélectionner un jeu.", "error");
            return;
        }

        const selectedGame = allGames.find(g => g.id === game_id);
        const title = selectedGame ? `Match ${selectedGame.name}` : "Match";

        try {
            await api.post('/api/admin/matches', {
                title,
                game_id,
                assigned_teams: checkedTeams,
                instructions,
                is_open: true
            });
            showToast("Match créé avec succès !", "success");
            e.target.reset();
            loadMatches();
        } catch (err) {
            showToast(err.message, "error");
        }
    });

    // 3. Soumission Saisie Groupée
    document.getElementById('bulk-scores-form')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const matchId = document.getElementById('bulk-match-id').value;
        const rows = document.querySelectorAll('#bulk-scores-tbody tr');

        const scores = [];
        rows.forEach(row => {
            const team_id = row.getAttribute('data-team-id');
            const placement = parseInt(row.querySelector('.bulk-input-placement').value, 10);
            const secondary_stat = parseInt(row.querySelector('.bulk-input-secondary').value, 10) || 0;
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
            showToast("Scores groupés enregistrés et points validés !", "success");
            closeModal('modal-bulk-scores');
            loadMatches();
            loadTeams();
        } catch (err) {
            showToast(err.message, "error");
        }
    });

    // 4. Création d'Équipe
    document.getElementById('create-team-form')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const name = document.getElementById('team-name').value.trim();
        const captain_password = document.getElementById('team-password').value;

        try {
            await api.post('/api/admin/teams', { name, captain_password });
            showToast(`Équipe "${name}" créée avec succès !`, "success");
            e.target.reset();
            loadTeams();
        } catch (err) {
            showToast(err.message, "error");
        }
    });

    // 5. Création d'Admin
    document.getElementById('create-admin-form')?.addEventListener('submit', async (e) => {
        e.preventDefault();
        const username = document.getElementById('new-admin-username').value.trim();
        const password = document.getElementById('new-admin-password').value;

        try {
            await api.post('/api/admin/admins', { username, password });
            showToast(`Compte admin "${username}" créé !`, "success");
            e.target.reset();
            loadAdmins();
        } catch (err) {
            showToast(err.message, "error");
        }
    });

    // 6. Zone de Danger (Reset)
    document.querySelectorAll('[data-reset-scope]').forEach(btn => {
        btn.addEventListener('click', () => {
            const rawScope = btn.getAttribute('data-reset-scope');
            const titleEl = document.getElementById('reset-scope-title');
            const descEl = document.getElementById('reset-scope-desc');
            const inputEl = document.getElementById('reset-confirm-input');
            const confirmBtn = document.getElementById('confirm-reset-execute-btn');

            inputEl.value = '';
            confirmBtn.disabled = true;

            if (rawScope === 'ScoresOnly') {
                activeResetScope = 'scores_only';
                titleEl.textContent = "Remise à zéro des Scores";
                descEl.textContent = "Attention : tous les scores et soumissions actuels seront supprimés. Les compteurs de points repartiront à zéro.";
            } else if (rawScope === 'UsersAndTeams') {
                activeResetScope = 'users_and_teams';
                titleEl.textContent = "Purge des Équipes & Joueurs";
                descEl.textContent = "Attention : toutes les équipes et leurs capitaines seront définitivement effacés.";
            } else {
                activeResetScope = 'full_tournament';
                titleEl.textContent = "Reset Total du Tournoi";
                descEl.textContent = "DANGER EXTRÊME : purge intégrale de la base de données. Seul votre compte administrateur sera préservé.";
            }

            openModal('modal-confirm-reset');
        });
    });

    const resetInput = document.getElementById('reset-confirm-input');
    const executeResetBtn = document.getElementById('confirm-reset-execute-btn');

    resetInput?.addEventListener('input', () => {
        executeResetBtn.disabled = resetInput.value.trim().toUpperCase() !== 'RESET';
    });

    executeResetBtn?.addEventListener('click', async () => {
        if (!activeResetScope) return;
        executeResetBtn.disabled = true;
        executeResetBtn.textContent = "Réinitialisation en cours...";

        try {
            await api.post('/api/admin/reset', { scope: activeResetScope });
            showToast("Réinitialisation effectuée avec succès.", "success");
            closeModal('modal-confirm-reset');

            setTimeout(() => {
                window.location.reload();
            }, 800);
        } catch (err) {
            showToast(err.message, "error");
            executeResetBtn.disabled = false;
            executeResetBtn.textContent = "Exécuter la Réinitialisation";
        }
    });
}
