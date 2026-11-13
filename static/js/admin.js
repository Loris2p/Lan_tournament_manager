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
    initTournamentSchedule();
    initBulkScoresForm();
    initResetZone();
});

function initWebSocket() {
    wsClient.connect();

    wsClient.on('NEW_SUBMISSION', async () => {
        showToast("Nouveau score en attente d'arbitrage !", "info");
        await refreshSubmissionsSilently();
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

function wireSubmissionMediaZoom() {
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
}

async function refreshSubmissionsSilently() {
    try {
        const res = await fetch(window.location.href);
        if (!res.ok) return;
        const html = await res.text();
        const doc = new DOMParser().parseFromString(html, 'text/html');

        const newList = doc.getElementById('submissions-list');
        const currentList = document.getElementById('submissions-list');
        if (newList && currentList) {
            currentList.innerHTML = newList.innerHTML;
            wireSubmissionMediaZoom();
        }

        const newBadge = doc.getElementById('pending-badge');
        const currentBadge = document.getElementById('pending-badge');
        if (newBadge && currentBadge) {
            currentBadge.textContent = newBadge.textContent;
        }
    } catch (e) {
        console.warn("Échec du rafraîchissement silencieux des soumissions:", e);
    }
}

function initSubmissions() {
    wireSubmissionMediaZoom();

    const listEl = document.getElementById('submissions-list');
    if (listEl) {
        listEl.addEventListener('click', (e) => {
            const btn = e.target.closest('.btn-edit-submission');
            if (!btn) return;
            const subId = btn.getAttribute('data-sub-id');
            const teamName = btn.getAttribute('data-team-name') || '';
            const placement = btn.getAttribute('data-placement') || '1';
            const secondaryStat = btn.getAttribute('data-secondary-stat') || '0';
            const proofUrl = btn.getAttribute('data-proof-url') || '';

            const form = document.getElementById('edit-submission-form');
            const titleEl = document.getElementById('edit-sub-team-title');
            const placementInput = document.getElementById('edit-sub-placement');
            const secondaryInput = document.getElementById('edit-sub-secondary');
            const proofPreview = document.getElementById('edit-sub-proof-preview');
            const proofImg = document.getElementById('edit-sub-proof-img');

            if (form && subId) {
                form.action = `/admin/submissions/${subId}/edit`;
            }
            if (titleEl) {
                titleEl.textContent = `Modifier la soumission — ${teamName}`;
            }
            if (placementInput) placementInput.value = placement;
            if (secondaryInput) secondaryInput.value = secondaryStat;
            if (proofPreview && proofImg) {
                if (proofUrl) {
                    proofImg.src = proofUrl;
                    proofPreview.style.display = 'block';
                } else {
                    proofImg.removeAttribute('src');
                    proofPreview.style.display = 'none';
                }
            }

            openModal('modal-edit-submission');
        });
    }

    document.getElementById('refresh-submissions-btn')?.addEventListener('click', async () => {
        await refreshSubmissionsSilently();
        showToast("File d'arbitrage synchronisée !", "info");
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
        btn.addEventListener('click', async () => {
            const id = btn.getAttribute('data-id');
            await openBulkScoresModal(id);
        });
    });

    const searchInput = document.getElementById('match-filter-search');
    const gameSelect = document.getElementById('match-filter-game');
    const statusSelect = document.getElementById('match-filter-status');
    const countBadge = document.getElementById('matches-count-badge');
    const emptyRow = document.getElementById('matches-filter-empty');
    const groups = Array.from(container.querySelectorAll('.match-game-group'));
    const rows = Array.from(container.querySelectorAll('.match-admin-row'));

    if (rows.length > 0 && (searchInput || gameSelect || statusSelect)) {
        const applyMatchFilters = () => {
            const query = (searchInput?.value || '').trim().toLowerCase();
            const game = gameSelect?.value || '';
            const status = statusSelect?.value || 'all';
            let visibleCount = 0;

            groups.forEach(group => {
                const groupRows = Array.from(group.querySelectorAll('.match-admin-row'));
                let groupVisible = 0;

                groupRows.forEach(row => {
                    const matchSearch = !query || (row.dataset.search || '').toLowerCase().includes(query);
                    const matchGame = !game || row.dataset.game === game;
                    const matchStatus = status === 'all' || row.dataset.status === status;
                    const visible = matchSearch && matchGame && matchStatus;
                    row.style.display = visible ? '' : 'none';
                    if (visible) {
                        visibleCount += 1;
                        groupVisible += 1;
                    }
                });

                group.style.display = groupVisible > 0 ? '' : 'none';
                const groupBadge = group.querySelector('.group-match-count');
                if (groupBadge) {
                    groupBadge.textContent = groupVisible === groupRows.length
                        ? `${groupRows.length}`
                        : `${groupVisible} / ${groupRows.length}`;
                }
            });

            if (countBadge) {
                countBadge.textContent = visibleCount === rows.length
                    ? `${rows.length}`
                    : `${visibleCount} / ${rows.length}`;
            }
            if (emptyRow) {
                emptyRow.style.display = visibleCount === 0 ? '' : 'none';
            }
        };

        searchInput?.addEventListener('input', applyMatchFilters);
        gameSelect?.addEventListener('change', applyMatchFilters);
        statusSelect?.addEventListener('change', applyMatchFilters);
    }
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

    const form = document.getElementById('bulk-scores-form');
    document.getElementById('bulk-match-id').value = match.id;
    document.getElementById('bulk-match-title').textContent = `Saisie Groupée : ${match.game_name || match.title || 'Match'}`;

    const tbody = document.getElementById('bulk-scores-tbody');
    const secHeader = document.getElementById('bulk-sec-header');
    const placementHeader = document.getElementById('bulk-placement-header');

    const game = cachedGames.find(g => g.id === match.game_id);
    const hasSec = Boolean(game?.secondary_stat_label);
    const teamPlayers = parseInt(game?.team_player_count || 1, 10);
    const maxPlacement = parseInt(game?.max_placement || 64, 10);
    const participatingTeams = cachedTeams.filter(t => (match.assigned_teams || []).includes(t.id));

    const isFallGuys = (game?.name || '').toLowerCase().includes('fall guys');
    const isDuel = !isFallGuys && (maxPlacement === 2 || (game?.format || '').toLowerCase() === 'duel') && participatingTeams.length === 2;
    const isScoreRanked = Boolean(hasSec && (game?.points_per_secondary_stat === 0) && (game?.placement_points || []).length > 0 && maxPlacement > 2);
    const hidePlacementCol = !isFallGuys && !isDuel && !isScoreRanked && maxPlacement <= 1;

    if (form) {
        form.setAttribute('data-is-score-ranked', isScoreRanked ? 'true' : 'false');
    }

    if (placementHeader) {
        if (hidePlacementCol) {
            placementHeader.style.display = 'none';
        } else {
            placementHeader.style.display = '';
            if (isFallGuys) {
                placementHeader.textContent = 'PRIME VICTOIRE SHOW';
            } else if (isDuel) {
                placementHeader.textContent = 'RÉSULTAT (1V1)';
            } else if (isScoreRanked) {
                placementHeader.textContent = 'CLASSEMENT (AUTO)';
            } else {
                placementHeader.textContent = 'CLASSEMENT (PLACE)';
            }
        }
    }

    if (secHeader) {
        if (isDuel && !hasSec) {
            secHeader.style.display = 'none';
        } else {
            secHeader.style.display = '';
            secHeader.textContent = hasSec 
                ? (isFallGuys 
                    ? `MANCHES FRANCHIES (${teamPlayers} JOUEURS)`
                    : (teamPlayers > 1 ? `${game.secondary_stat_label.toUpperCase()} (${teamPlayers} JOUEURS)` : game.secondary_stat_label.toUpperCase()))
                : 'STAT SECONDAIRE (N/A)';
        }
    }

    if (participatingTeams.length === 0) {
        tbody.innerHTML = `<tr><td colspan="3" class="cell-loading">Aucune équipe assignée à ce match.</td></tr>`;
    } else {
        tbody.innerHTML = participatingTeams.map((t, idx) => {
            let placementCellHtml = '';
            if (isFallGuys) {
                placementCellHtml = `
                    <td>
                        <div class="bulk-duel-buttons">
                            <button type="button" class="btn-outcome btn-outcome-win" data-val="1">
                                <span class="papicon icon-crown" aria-hidden="true"></span> Gagné (+3)
                            </button>
                            <button type="button" class="btn-outcome btn-outcome-loss active" data-val="2">
                                <span class="papicon icon-cross" aria-hidden="true"></span> Non (+0)
                            </button>
                            <input type="hidden" class="bulk-input-placement" value="2">
                        </div>
                    </td>
                `;
            } else if (isDuel) {
                placementCellHtml = `
                    <td>
                        <div class="bulk-duel-buttons">
                            <button type="button" class="btn-outcome btn-outcome-win ${idx === 0 ? 'active' : ''}" data-val="1">
                                <span class="papicon icon-crown" aria-hidden="true"></span> Victoire
                            </button>
                            <button type="button" class="btn-outcome btn-outcome-loss ${idx === 1 ? 'active' : ''}" data-val="2">
                                <span class="papicon icon-cross" aria-hidden="true"></span> Défaite
                            </button>
                            <input type="hidden" class="bulk-input-placement" value="${idx === 0 ? '1' : '2'}">
                        </div>
                    </td>
                `;
            } else if (isScoreRanked) {
                placementCellHtml = `
                    <td>
                        <span class="badge-format" style="color: var(--text-muted); font-size: 0.75rem;">Auto (par score)</span>
                        <input type="hidden" class="bulk-input-placement" value="${idx + 1}">
                    </td>
                `;
            } else if (hidePlacementCol) {
                placementCellHtml = `
                    <td style="display: none;">
                        <input type="hidden" class="bulk-input-placement" value="1">
                    </td>
                `;
            } else {
                placementCellHtml = `
                    <td>
                        <input type="number" class="bulk-input-placement" min="1" max="${maxPlacement}" value="${idx + 1}" required style="width: 80px;">
                    </td>
                `;
            }

            let secCellHtml = '';
            if (isDuel && !hasSec) {
                secCellHtml = `<td style="display: none;"><input type="hidden" class="bulk-input-secondary" value="0"></td>`;
            } else if (hasSec && teamPlayers > 1) {
                secCellHtml = `
                    <td>
                        <div class="bulk-multi-inputs">
                            ${Array.from({ length: teamPlayers }, (_, pIdx) => `
                                <input type="number" class="bulk-player-input" data-player="${pIdx + 1}" placeholder="J${pIdx + 1}" min="0" value="0">
                            `).join('')}
                            <span class="bulk-team-sum badge-code">${isFallGuys ? 'Manches : 0' : 'Total : 0'}</span>
                            <input type="hidden" class="bulk-input-secondary" value="0">
                        </div>
                    </td>
                `;
            } else {
                secCellHtml = `
                    <td>
                        <input type="number" class="bulk-input-secondary" min="0" value="0" ${hasSec ? '' : 'disabled'} style="width: 80px;">
                    </td>
                `;
            }

            return `
                <tr data-team-id="${t.id}">
                    <td><strong>${escapeHtml(t.name)}</strong></td>
                    ${placementCellHtml}
                    ${secCellHtml}
                </tr>
            `;
        }).join('');

        wireBulkEvents(tbody, isDuel, teamPlayers, isFallGuys);
    }

    openModal('modal-bulk-scores');
}

function wireBulkEvents(tbody, isDuel, teamPlayers, isFallGuys) {
    if (isFallGuys) {
        const rows = tbody.querySelectorAll('tr');
        tbody.querySelectorAll('.btn-outcome').forEach(btn => {
            btn.addEventListener('click', () => {
                const row = btn.closest('tr');
                const val = btn.getAttribute('data-val');

                if (val === '1') {
                    // Une seule équipe peut remporter la prime de victoire finale
                    rows.forEach(r => {
                        const input = r.querySelector('.bulk-input-placement');
                        const isTarget = (r === row);
                        if (input) input.value = isTarget ? '1' : '2';
                        r.querySelectorAll('.btn-outcome').forEach(b => {
                            b.classList.toggle('active', isTarget ? b.getAttribute('data-val') === '1' : b.getAttribute('data-val') === '2');
                        });
                    });
                } else {
                    // Annuler la victoire sur cette équipe
                    const input = row.querySelector('.bulk-input-placement');
                    if (input) input.value = '2';
                    row.querySelectorAll('.btn-outcome').forEach(b => {
                        b.classList.toggle('active', b.getAttribute('data-val') === '2');
                    });
                }
            });
        });
    } else if (isDuel) {
        const rows = tbody.querySelectorAll('tr');
        tbody.querySelectorAll('.btn-outcome').forEach(btn => {
            btn.addEventListener('click', () => {
                const row = btn.closest('tr');
                const val = btn.getAttribute('data-val');
                const oppositeVal = val === '1' ? '2' : '1';

                row.querySelector('.bulk-input-placement').value = val;
                row.querySelectorAll('.btn-outcome').forEach(b => {
                    b.classList.toggle('active', b.getAttribute('data-val') === val);
                });

                rows.forEach(otherRow => {
                    if (otherRow !== row) {
                        const input = otherRow.querySelector('.bulk-input-placement');
                        if (input) input.value = oppositeVal;
                        otherRow.querySelectorAll('.btn-outcome').forEach(b => {
                            b.classList.toggle('active', b.getAttribute('data-val') === oppositeVal);
                        });
                    }
                });
            });
        });
    }

    if (teamPlayers > 1) {
        tbody.querySelectorAll('.bulk-player-input').forEach(input => {
            input.addEventListener('input', () => {
                const row = input.closest('tr');
                const playerInputs = row.querySelectorAll('.bulk-player-input');
                let sum = 0;
                playerInputs.forEach(inp => {
                    sum += (parseInt(inp.value, 10) || 0);
                });
                const totalEl = row.querySelector('.bulk-team-sum');
                const secInput = row.querySelector('.bulk-input-secondary');
                if (totalEl) totalEl.textContent = isFallGuys ? `Manches : ${sum}` : `Total : ${sum}`;
                if (secInput) secInput.value = sum;
            });
        });
    }
}

function initBulkScoresForm() {
    const form = document.getElementById('bulk-scores-form');
    if (!form) return;

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const matchId = document.getElementById('bulk-match-id').value;
        const rows = document.querySelectorAll('#bulk-scores-tbody tr');
        const isScoreRanked = form.getAttribute('data-is-score-ranked') === 'true';

        const scores = [];
        rows.forEach(row => {
            const team_id = row.getAttribute('data-team-id');
            const placement = parseInt(row.querySelector('.bulk-input-placement')?.value, 10) || 1;
            const secondary_stat = parseInt(row.querySelector('.bulk-input-secondary')?.value, 10) || 0;
            if (team_id) {
                scores.push({
                    team_id,
                    score: { placement, secondary_stat }
                });
            }
        });

        if (isScoreRanked) {
            scores.sort((a, b) => b.score.secondary_stat - a.score.secondary_stat);
            scores.forEach((entry, idx) => {
                entry.score.placement = idx + 1;
            });
        }

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
                    input.value = String(adjustment);
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

            let scopeVal;
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

    api.get('/api/admin/settings')
        .then(settings => {
            if (settings && Array.isArray(settings.global_placement_points)) {
                input.value = settings.global_placement_points.join(', ');
            }
        })
        .catch(err => {
            console.warn("Impossible de charger les paramètres de tournoi:", err);
        });

    defaultBtn?.addEventListener('click', () => {
        input.value = '25, 18, 15, 12, 10, 8, 6, 4, 2, 1';
        showToast("Barème standard par défaut (10 équipes) chargé.", "info");
    });

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
                statusMsg.innerHTML = '<span class="papicon icon-check" aria-hidden="true"></span> Enregistré';
                statusMsg.style.color = "var(--green, #22c55e)";
                setTimeout(() => { statusMsg.textContent = ""; }, 3000);
            }
        } catch (err) {
            showToast(err.message, "error");
        }
    });
}

function initTournamentSchedule() {
    const form = document.getElementById('tournament-schedule-form');
    const startDateInput = document.getElementById('schedule-start-date');
    const startTimeInput = document.getElementById('schedule-start-time');
    const endDateInput = document.getElementById('schedule-end-date');
    const endTimeInput = document.getElementById('schedule-end-time');
    const pausedCheckbox = document.getElementById('schedule-is-paused');
    const statusMsg = document.getElementById('schedule-status-msg');

    const btnPresetWeekend = document.getElementById('schedule-preset-weekend');
    const btnPreset48h = document.getElementById('schedule-preset-48h');
    const btnPresetClear = document.getElementById('schedule-preset-clear');

    if (!form || !startDateInput || !endDateInput) return;

    const parseIso = (isoStr) => {
        if (!isoStr) return { date: '', time: '18:00' };
        const parts = isoStr.split('T');
        const date = parts[0] || '';
        let time = '18:00';
        if (parts[1]) {
            time = parts[1].slice(0, 5);
        }
        return { date, time };
    };

    const combineIso = (dateInput, timeInput) => {
        const d = dateInput.value.trim();
        if (!d) return null;
        const t = (timeInput.value.trim()) || '18:00';
        return `${d}T${t}`;
    };

    const pad = (n) => String(n).padStart(2, '0');
    const formatYmd = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;

    // Raccourci Prochain Week-end (Vendredi 18h -> Dimanche 18h)
    btnPresetWeekend?.addEventListener('click', () => {
        const now = new Date();
        const day = now.getDay(); // 0: Dimanche, 5: Vendredi
        const daysUntilFriday = (5 - day + 7) % 7 || 7;
        const friday = new Date(now.getFullYear(), now.getMonth(), now.getDate() + daysUntilFriday);
        const sunday = new Date(friday.getFullYear(), friday.getMonth(), friday.getDate() + 2);

        startDateInput.value = formatYmd(friday);
        startTimeInput.value = '18:00';
        endDateInput.value = formatYmd(sunday);
        endTimeInput.value = '18:00';
        showToast("Dates du prochain week-end pré-remplies (Ven 18h → Dim 18h).", "info");
    });

    // Raccourci Début + 48h
    btnPreset48h?.addEventListener('click', () => {
        if (!startDateInput.value) {
            showToast("Veuillez d'abord renseigner la date de lancement.", "error");
            return;
        }
        const start = new Date(startDateInput.value + 'T00:00:00');
        const end = new Date(start.getTime() + 48 * 3600 * 1000);
        endDateInput.value = formatYmd(end);
        endTimeInput.value = startTimeInput.value || '18:00';
        showToast("Clôture calculée à Début + 48 heures.", "info");
    });

    // Raccourci Effacer
    btnPresetClear?.addEventListener('click', () => {
        startDateInput.value = '';
        endDateInput.value = '';
        startTimeInput.value = '18:00';
        endTimeInput.value = '18:00';
        showToast("Dates effacées. N'oubliez pas d'enregistrer.", "info");
    });

    api.get('/api/admin/settings')
        .then(settings => {
            if (settings) {
                if (settings.start_at) {
                    const parsed = parseIso(settings.start_at);
                    startDateInput.value = parsed.date;
                    startTimeInput.value = parsed.time;
                }
                if (settings.end_at) {
                    const parsed = parseIso(settings.end_at);
                    endDateInput.value = parsed.date;
                    endTimeInput.value = parsed.time;
                }
                if (pausedCheckbox) pausedCheckbox.checked = Boolean(settings.is_paused);
            }
        })
        .catch(err => {
            console.warn("Impossible de charger le planning du tournoi:", err);
        });

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        try {
            await api.post('/api/admin/settings', {
                start_at: combineIso(startDateInput, startTimeInput),
                end_at: combineIso(endDateInput, endTimeInput),
                is_paused: pausedCheckbox ? pausedCheckbox.checked : false,
            });
            showToast("Planning et cycle de vie du tournoi enregistrés !", "success");
            if (statusMsg) {
                statusMsg.innerHTML = '<span class="papicon icon-check" aria-hidden="true"></span> Enregistré';
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
            teamPlayers: 3,
            rules: "Salon personnalisé Solo (3 joueurs/équipe). +1 pt par manche franchie par joueur. Prime de victoire de 3 pts en cas de victoire du show (place 1)."
        },
        geoguessr: {
            name: "GeoGuessr",
            format: "brawl",
            cover: "",
            placement: "25, 18, 15, 12, 10, 8, 6, 4, 2, 1",
            defPoints: 0,
            maxPlacement: 10,
            secLabel: "Score brut de la manche (départage)",
            secPts: 0,
            maxSec: 75000,
            teamPlayers: 3,
            rules: "Challenge individuel 3 joueurs/équipe. Mode No Move 1m15/round. Cumul des 3 scores bruts GeoGuessr pour déterminer le classement de la manche. Attribution des points de placement du barème général (25, 18, 15...)."
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
            teamPlayers: 3,
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
            const teamPlayersEl = document.getElementById('game-team-players');
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
            if (teamPlayersEl) teamPlayersEl.value = data.teamPlayers || '';
            if (rulesEl) rulesEl.value = data.rules;

            showToast(`Préréglage "${data.name}" chargé dans le formulaire !`, "info");
        });
    });
}
