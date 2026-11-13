import { wsClient } from './ws.js';
import { showToast, openModal, closeModal } from './ui.js';

let selectedMatch = null;
let playerSubmissionsData = [];
let activePlayerIndex = 1;
let currentTeamPlayerCount = 1;

document.addEventListener('DOMContentLoaded', () => {
    initMatches();
    initDuelButtons();
    initDropzoneAndPaste();
    initFormSubmit();
    initTournamentCountdown();

    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', async () => {
        await updateCaptainPointsSilently();
        if (isFormDirty()) {
            showToast("Classement mis à jour (votre saisie en cours est préservée)", "info");
        } else {
            window.location.reload();
        }
    });

    wsClient.on('TOURNAMENT_SCHEDULE_UPDATE', () => {
        window.location.reload();
    });

    wsClient.on('MATCH_UPDATE', async () => {
        await handleCaptainMatchUpdate();
    });

    const newSubBtn = document.getElementById('new-submission-btn');
    if (newSubBtn) {
        newSubBtn.addEventListener('click', () => {
            const successBox = document.getElementById('submission-success');
            const form = document.getElementById('submission-form');
            if (successBox) successBox.style.display = 'none';
            if (form) form.style.display = 'block';
            resetImage();
        });
    }
});

function initMatches() {
    const items = document.querySelectorAll('#matches-list .match-item');
    items.forEach(item => {
        item.addEventListener('click', () => {
            items.forEach(i => i.classList.remove('selected'));
            item.classList.add('selected');
            selectMatchElement(item);
        });
    });

    if (items.length > 0) {
        selectMatchElement(items[0]);
    }
}

function initDuelButtons() {
    const duelGroup = document.getElementById('duel-placement-group');
    if (!duelGroup) return;

    duelGroup.querySelectorAll('.btn-outcome').forEach(btn => {
        btn.addEventListener('click', () => {
            const val = btn.getAttribute('data-value');
            const placementInput = document.getElementById('sub-placement');
            if (placementInput) {
                placementInput.value = val;
            }
            duelGroup.querySelectorAll('.btn-outcome').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
        });
    });
}

function selectMatchElement(el) {
    const matchId = el.getAttribute('data-match-id');
    const title = el.getAttribute('data-title') || 'FORMAT STANDARD';
    const game = el.getAttribute('data-game') || 'Match';
    const instructions = el.getAttribute('data-instructions') || 'Aucune consigne spécifique.';
    const rules = el.getAttribute('data-rules') || 'Règles standards du catalogue.';
    const format = el.getAttribute('data-format') || 'Brawl';
    const secondaryLabel = el.getAttribute('data-secondary-label') || '';
    const secondaryPoints = parseInt(el.getAttribute('data-secondary-points') || '0', 10);
    const maxPlacement = parseInt(el.getAttribute('data-max-placement') || '64', 10);
    const maxSecondary = parseInt(el.getAttribute('data-max-secondary') || '100', 10);
    currentTeamPlayerCount = parseInt(el.getAttribute('data-team-player-count') || '1', 10);

    const isFallGuys = game.toLowerCase().includes('fall guys');
    const isScoreRanked = Boolean(secondaryLabel && secondaryLabel.trim().length > 0 && secondaryPoints === 0 && maxPlacement > 2);
    const noPlacementNeeded = isScoreRanked || maxPlacement <= 1;
    selectedMatch = { id: matchId, title, game_name: game, instructions, maxPlacement, isScoreRanked, isFallGuys, noPlacementNeeded };

    const detailsPanel = document.getElementById('selected-match-details');
    const noMatchAlert = document.getElementById('no-match-alert');
    const form = document.getElementById('submission-form');
    const subMatchId = document.getElementById('sub-match-id');

    if (detailsPanel) detailsPanel.style.display = 'block';
    if (noMatchAlert) noMatchAlert.style.display = 'none';
    if (form) form.style.display = 'block';
    if (subMatchId) subMatchId.value = matchId;

    const gameEl = document.getElementById('selected-match-game');
    if (gameEl) gameEl.textContent = game;
    const formatEl = document.getElementById('selected-match-format');
    if (formatEl) formatEl.textContent = `${title} [${format.toUpperCase()}]`;
    const instEl = document.getElementById('selected-match-instructions');
    if (instEl) instEl.textContent = instructions;
    const rulesEl = document.getElementById('selected-match-rules');
    if (rulesEl) rulesEl.textContent = rules;

    const placementGroup = document.getElementById('placement-group');
    const placementInput = document.getElementById('sub-placement');
    const placementHint = document.getElementById('placement-hint');
    const placementLabel = document.getElementById('placement-label');
    const duelPlacementGroup = document.getElementById('duel-placement-group');
    const standardPlacementGroup = document.getElementById('standard-placement-group');

    if (noPlacementNeeded) {
        if (placementGroup) placementGroup.style.display = 'none';
        if (placementInput) placementInput.value = '1';
    } else if (isFallGuys) {
        if (placementGroup) placementGroup.style.display = 'block';
        if (duelPlacementGroup) duelPlacementGroup.style.display = 'grid';
        if (standardPlacementGroup) standardPlacementGroup.style.display = 'none';
        if (placementLabel) placementLabel.textContent = "PRIME DE VICTOIRE DU SHOW";
        
        const winBtn = duelPlacementGroup.querySelector('.btn-outcome-win');
        const lossBtn = duelPlacementGroup.querySelector('.btn-outcome-loss');
        if (winBtn) winBtn.innerHTML = '<span class="papicon icon-crown" aria-hidden="true"></span> Show Remporté (+3 pts)';
        if (lossBtn) lossBtn.innerHTML = '<span class="papicon icon-cross" aria-hidden="true"></span> Pas de victoire (+0 pt)';
        
        if (placementInput) placementInput.value = '2';
        if (lossBtn) lossBtn.classList.add('active');
        if (winBtn) winBtn.classList.remove('active');
    } else {
        if (placementGroup) placementGroup.style.display = 'block';
        if (maxPlacement === 2) {
            if (duelPlacementGroup) duelPlacementGroup.style.display = 'grid';
            if (standardPlacementGroup) standardPlacementGroup.style.display = 'none';
            if (placementLabel) placementLabel.textContent = "RÉSULTAT DU MATCH";
            
            const winBtn = duelPlacementGroup.querySelector('.btn-outcome-win');
            const lossBtn = duelPlacementGroup.querySelector('.btn-outcome-loss');
            if (winBtn) winBtn.innerHTML = '<span class="papicon icon-crown" aria-hidden="true"></span> Victoire';
            if (lossBtn) lossBtn.innerHTML = '<span class="papicon icon-cross" aria-hidden="true"></span> Défaite';
            
            if (placementInput) placementInput.value = '';
            if (duelPlacementGroup) {
                duelPlacementGroup.querySelectorAll('.btn-outcome').forEach(b => b.classList.remove('active'));
            }
        } else {
            if (duelPlacementGroup) duelPlacementGroup.style.display = 'none';
            if (standardPlacementGroup) standardPlacementGroup.style.display = 'block';
            if (placementLabel) placementLabel.textContent = "CLASSEMENT FINAL (PLACE)";
            if (placementInput) {
                placementInput.value = '';
                placementInput.max = maxPlacement;
                placementInput.placeholder = `Ex: 1 (Top 1), 2... (max ${maxPlacement})`;
                if (placementHint) placementHint.textContent = `Place finale obtenue dans le salon (1 à ${maxPlacement})`;
            }
        }
    }

    const secGroup = document.getElementById('secondary-stat-group');
    const secLabel = document.getElementById('secondary-stat-label');
    const secInput = document.getElementById('sub-secondary');
    const secHint = document.getElementById('secondary-stat-hint');

    if (secondaryLabel && secondaryLabel.trim().length > 0) {
        if (secGroup) secGroup.style.display = 'block';
        if (secLabel) secLabel.textContent = secondaryLabel.toUpperCase();
        if (secInput) {
            secInput.max = maxSecondary;
        }
        if (secHint) {
            secHint.textContent = secondaryPoints > 0 
                ? `+${secondaryPoints} pt(s) par unité (Max ${maxSecondary})`
                : `Max autorisé : ${maxSecondary}`;
        }
    } else {
        if (secGroup) secGroup.style.display = 'none';
        if (secInput) secInput.value = '0';
    }

    const multiContainer = document.getElementById('multi-players-container');
    const singleDropzone = document.getElementById('single-dropzone-group');

    if (currentTeamPlayerCount > 1) {
        if (multiContainer) multiContainer.style.display = 'block';
        if (singleDropzone) singleDropzone.style.display = 'none';
        playerSubmissionsData = Array.from({ length: currentTeamPlayerCount }, (_, i) => ({
            player_number: i + 1,
            score: 0,
            proof_base64: ''
        }));
        renderMultiPlayerCards();
        if (secInput) {
            secInput.readOnly = true;
            secInput.value = '0';
        }
        if (secHint) {
            secHint.textContent = `Calcul automatique (${currentTeamPlayerCount} joueurs)`;
        }
    } else {
        if (multiContainer) multiContainer.style.display = 'none';
        if (singleDropzone) singleDropzone.style.display = 'block';
        playerSubmissionsData = [];
        const jsonInput = document.getElementById('sub-player-submissions-json');
        if (jsonInput) jsonInput.value = '';
        if (secInput) secInput.readOnly = false;
    }
}

function compressImageFile(file) {
    return new Promise((resolve, reject) => {
        if (!file || !file.type.startsWith('image/')) {
            return reject(new Error("Le fichier doit être une image valide."));
        }
        const reader = new FileReader();
        reader.onerror = reject;
        reader.onload = (event) => {
            const img = new Image();
            img.onerror = reject;
            img.onload = () => {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');

                const MAX_WIDTH = 1920;
                const MAX_HEIGHT = 1080;
                let width = img.width;
                let height = img.height;

                if (width > height) {
                    if (width > MAX_WIDTH) {
                        height = Math.round(height * (MAX_WIDTH / width));
                        width = MAX_WIDTH;
                    }
                } else {
                    if (height > MAX_HEIGHT) {
                        width = Math.round(width * (MAX_HEIGHT / height));
                        height = MAX_HEIGHT;
                    }
                }

                canvas.width = width;
                canvas.height = height;
                ctx.drawImage(img, 0, 0, width, height);

                const dataUrl = canvas.toDataURL('image/jpeg', 0.80);
                resolve(dataUrl);
            };
            if (typeof event.target?.result === 'string') {
                img.src = event.target.result;
            }
        };
        reader.readAsDataURL(file);
    });
}

function renderMultiPlayerCards() {
    const listEl = document.getElementById('multi-players-list');
    if (!listEl) return;

    listEl.innerHTML = playerSubmissionsData.map(p => `
        <div class="player-card" data-player="${p.player_number}" style="background: rgba(255,255,255,0.03); border: 1px solid var(--border-color, #334155); border-radius: 6px; padding: 0.85rem;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                <strong style="color: var(--text-main, #f8fafc); font-size: 0.95rem;">Joueur ${p.player_number}</strong>
                <span class="player-status-badge badge-code" id="player-status-${p.player_number}" style="color: var(--text-muted, #94a3b8); font-size: 0.75rem;">${p.proof_base64 ? '✓ Photo prête' : 'En attente de photo'}</span>
            </div>
            <div style="display: grid; grid-template-columns: 1fr 1.5fr; gap: 0.75rem; align-items: start;">
                <div>
                    <label style="font-size: 0.75rem; color: var(--text-muted); display: block; margin-bottom: 0.25rem;">
                        ${selectedMatch?.isFallGuys ? 'MANCHES FRANCHIES (SURVIE)' : 'SCORE DU JOUEUR'}
                    </label>
                    <input type="number" class="player-score-input" data-player="${p.player_number}" min="0" value="${p.score || 0}" required style="width: 100%;">
                </div>
                <div>
                    <label style="font-size: 0.75rem; color: var(--text-muted); display: block; margin-bottom: 0.25rem;">CAPTURE / PHOTO DE PREUVE</label>
                    <div class="player-dropzone" data-player="${p.player_number}" tabindex="0" style="border: 1px dashed var(--border-color, #475569); border-radius: 4px; padding: 0.5rem; text-align: center; background: rgba(0,0,0,0.15); cursor: pointer;">
                        <div id="player-prompt-${p.player_number}" ${p.proof_base64 ? 'style="display:none;"' : ''}>
                            <div style="display: flex; gap: 0.4rem; justify-content: center; flex-wrap: wrap;">
                                <label for="player-cam-${p.player_number}" class="btn-outline btn-sm" style="cursor: pointer; font-size: 0.75rem; padding: 0.2rem 0.4rem;">📷 Photo Mobile</label>
                                <label for="player-file-${p.player_number}" class="btn-text btn-sm" style="cursor: pointer; font-size: 0.75rem; padding: 0.2rem 0.4rem;">Fichier</label>
                            </div>
                            <span style="font-size: 0.7rem; color: var(--text-muted); display: block; margin-top: 0.25rem;">ou coller (Ctrl+V)</span>
                            <input type="file" id="player-file-${p.player_number}" accept="image/*" style="display: none;">
                            <input type="file" id="player-cam-${p.player_number}" accept="image/*" capture="environment" style="display: none;">
                        </div>
                        <div id="player-preview-${p.player_number}" style="${p.proof_base64 ? 'display:block;' : 'display:none;'}">
                            <img src="${p.proof_base64 || ''}" id="player-img-${p.player_number}" alt="Capture J${p.player_number}" style="max-height: 80px; width: 100%; object-fit: contain; border-radius: 3px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 0.25rem;">
                                <span class="badge-code" style="font-size: 0.7rem; color: var(--green, #22c55e);"><span class="papicon icon-check" aria-hidden="true"></span> Prête</span>
                                <button type="button" class="btn-text btn-sm btn-clear-player" data-player="${p.player_number}" style="font-size: 0.7rem; color: var(--red, #ef4444);">Remplacer</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `).join('');

    wireMultiPlayerEvents();
}

function wireMultiPlayerEvents() {
    const listEl = document.getElementById('multi-players-list');
    if (!listEl) return;

    listEl.querySelectorAll('.player-score-input').forEach(input => {
        input.addEventListener('input', () => {
            const num = parseInt(input.getAttribute('data-player'), 10);
            const val = parseInt(input.value, 10) || 0;
            const target = playerSubmissionsData.find(p => p.player_number === num);
            if (target) target.score = val;
            updateMultiPlayerSum();
        });
    });

    listEl.querySelectorAll('.player-dropzone').forEach(zone => {
        const num = parseInt(zone.getAttribute('data-player'), 10);
        zone.addEventListener('click', () => {
            activePlayerIndex = num;
        });
        zone.addEventListener('dragover', (e) => {
            e.preventDefault();
            zone.style.borderColor = 'var(--accent, #e53e3e)';
        });
        zone.addEventListener('dragleave', () => {
            zone.style.borderColor = 'var(--border-color, #475569)';
        });
        zone.addEventListener('drop', async (e) => {
            e.preventDefault();
            zone.style.borderColor = 'var(--border-color, #475569)';
            if (e.dataTransfer.files && e.dataTransfer.files[0]) {
                await handlePlayerFile(num, e.dataTransfer.files[0]);
            }
        });
    });

    playerSubmissionsData.forEach(p => {
        const fileInput = document.getElementById(`player-file-${p.player_number}`);
        const camInput = document.getElementById(`player-cam-${p.player_number}`);
        fileInput?.addEventListener('change', async (e) => {
            if (e.target.files && e.target.files[0]) {
                await handlePlayerFile(p.player_number, e.target.files[0]);
            }
        });
        camInput?.addEventListener('change', async (e) => {
            if (e.target.files && e.target.files[0]) {
                await handlePlayerFile(p.player_number, e.target.files[0]);
            }
        });
    });

    listEl.querySelectorAll('.btn-clear-player').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const num = parseInt(btn.getAttribute('data-player'), 10);
            clearPlayerImage(num);
        });
    });

    updateMultiPlayerSum();
}

function updateMultiPlayerSum() {
    const sum = playerSubmissionsData.reduce((acc, p) => acc + (p.score || 0), 0);
    const badge = document.getElementById('multi-players-calc-badge');
    const secInput = document.getElementById('sub-secondary');
    if (badge) {
        badge.textContent = selectedMatch?.isFallGuys ? `MANCHES CUMULÉES : ${sum}` : `CUMUL : ${sum}`;
    }
    if (secInput) secInput.value = sum;
}

async function handlePlayerFile(playerNumber, file) {
    if (!file || !file.type.startsWith('image/')) {
        showToast("Le fichier sélectionné doit être une image.", "error");
        return;
    }
    try {
        const dataUrl = await compressImageFile(file);
        const target = playerSubmissionsData.find(p => p.player_number === playerNumber);
        if (target) target.proof_base64 = dataUrl;

        const prompt = document.getElementById(`player-prompt-${playerNumber}`);
        const preview = document.getElementById(`player-preview-${playerNumber}`);
        const img = document.getElementById(`player-img-${playerNumber}`);
        const status = document.getElementById(`player-status-${playerNumber}`);

        if (prompt) prompt.style.display = 'none';
        if (preview) preview.style.display = 'block';
        if (img) img.src = dataUrl;
        if (status) {
            status.innerHTML = `<span class="papicon icon-check" aria-hidden="true"></span> Photo prête`;
            status.style.color = "var(--green, #22c55e)";
        }
        showToast(`Capture du Joueur ${playerNumber} enregistrée !`, "info");
    } catch (err) {
        showToast("Erreur lors de la compression de la capture : " + err.message, "error");
    }
}

function clearPlayerImage(playerNumber) {
    const target = playerSubmissionsData.find(p => p.player_number === playerNumber);
    if (target) target.proof_base64 = '';

    const prompt = document.getElementById(`player-prompt-${playerNumber}`);
    const preview = document.getElementById(`player-preview-${playerNumber}`);
    const img = document.getElementById(`player-img-${playerNumber}`);
    const status = document.getElementById(`player-status-${playerNumber}`);
    const fileInput = document.getElementById(`player-file-${playerNumber}`);
    const camInput = document.getElementById(`player-cam-${playerNumber}`);

    if (prompt) prompt.style.display = 'block';
    if (preview) preview.style.display = 'none';
    if (img) img.src = '';
    if (status) {
        status.textContent = "En attente de photo";
        status.style.color = "var(--text-muted, #94a3b8)";
    }
    if (fileInput) fileInput.value = '';
    if (camInput) camInput.value = '';
}

function initDropzoneAndPaste() {
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const clearBtn = document.getElementById('clear-image-btn');

    if (!dropzone || !fileInput) return;

    window.addEventListener('paste', async (e) => {
        if (!selectedMatch) return;
        const items = e.clipboardData?.items;
        if (!items) return;

        for (const item of items) {
            if (item.type.includes('image')) {
                const file = item.getAsFile();
                if (currentTeamPlayerCount > 1) {
                    const targetPlayer = activePlayerIndex || 1;
                    await handlePlayerFile(targetPlayer, file);
                } else {
                    processAndCompressImage(file);
                    showToast("Capture d'écran collée depuis le presse-papier !", "info");
                }
                break;
            }
        }
    });

    fileInput.addEventListener('change', (e) => {
        if (e.target.files && e.target.files[0]) {
            processAndCompressImage(e.target.files[0]);
        }
    });

    const cameraInput = document.getElementById('camera-input');
    if (cameraInput) {
        cameraInput.addEventListener('change', (e) => {
            if (e.target.files && e.target.files[0]) {
                processAndCompressImage(e.target.files[0]);
            }
        });
    }

    dropzone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropzone.classList.add('drag-active');
    });

    dropzone.addEventListener('dragleave', () => {
        dropzone.classList.remove('drag-active');
    });

    dropzone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropzone.classList.remove('drag-active');
        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
            processAndCompressImage(e.dataTransfer.files[0]);
        }
    });

    if (clearBtn) {
        clearBtn.addEventListener('click', resetImage);
    }
}

function resetImage() {
    const proofInput = document.getElementById('sub-proof-base64');
    const fileInput = document.getElementById('file-input');
    const cameraInput = document.getElementById('camera-input');
    const previewWrapper = document.getElementById('preview-wrapper');
    const dropzonePrompt = document.getElementById('dropzone-prompt');

    if (proofInput) proofInput.value = '';
    if (fileInput) fileInput.value = '';
    if (cameraInput) cameraInput.value = '';
    if (previewWrapper) previewWrapper.style.display = 'none';
    if (dropzonePrompt) dropzonePrompt.style.display = 'block';
}

async function processAndCompressImage(file) {
    if (!file || !file.type.startsWith('image/')) {
        showToast("Le fichier sélectionné n'est pas une image valide.", "error");
        return;
    }

    const originalSizeKb = Math.round(file.size / 1024);
    try {
        const dataUrl = await compressImageFile(file);
        const proofInput = document.getElementById('sub-proof-base64');
        if (proofInput) proofInput.value = dataUrl;

        const sizeKb = Math.round((dataUrl.length * 3 / 4) / 1024);
        const previewImg = document.getElementById('preview-img');
        const previewBadge = document.getElementById('preview-size-badge');

        if (previewImg) previewImg.src = dataUrl;
        if (previewBadge) {
            if (originalSizeKb > sizeKb) {
                const ratio = Math.round((1 - sizeKb / originalSizeKb) * 100);
                previewBadge.innerHTML = `<span class="papicon icon-check" aria-hidden="true"></span> Optimisée : ${originalSizeKb} Ko → ${sizeKb} Ko (-${ratio}%)`;
            } else {
                previewBadge.textContent = `${sizeKb} Ko [Compressé]`;
            }
        }

        const prompt = document.getElementById('dropzone-prompt');
        const wrapper = document.getElementById('preview-wrapper');
        if (prompt) prompt.style.display = 'none';
        if (wrapper) wrapper.style.display = 'block';
    } catch (err) {
        showToast("Erreur lors de la compression de l'image : " + err.message, "error");
    }
}

function initFormSubmit() {
    const form = document.getElementById('submission-form');
    const submitBtn = document.getElementById('submit-score-btn');
    const finalConfirmBtn = document.getElementById('btn-final-confirm-submit');
    if (!form || !submitBtn) return;

    document.querySelectorAll('[data-close-modal="modal-confirm-submission"]').forEach(btn => {
        btn.addEventListener('click', () => closeModal('modal-confirm-submission'));
    });

    let isConfirmed = false;

    form.addEventListener('submit', (e) => {
        if (isConfirmed) return; // Soumission autorisée suite à la modale

        e.preventDefault();

        if (!selectedMatch) {
            showToast("Veuillez sélectionner un match.", "error");
            return;
        }

        const placementInput = document.getElementById('sub-placement');
        if (!placementInput || !placementInput.value.trim()) {
            showToast(
                selectedMatch?.maxPlacement === 2
                    ? "Veuillez sélectionner le résultat (Victoire ou Défaite)."
                    : "Veuillez indiquer le classement final obtenu.",
                "error"
            );
            placementInput?.focus();
            return;
        }

        const proofInput = document.getElementById('sub-proof-base64');
        if (currentTeamPlayerCount > 1) {
            const missing = playerSubmissionsData.find(p => !p.proof_base64);
            if (missing) {
                showToast(`Veuillez joindre la capture d'écran pour le Joueur ${missing.player_number}.`, "error");
                return;
            }
            const jsonInput = document.getElementById('sub-player-submissions-json');
            if (jsonInput) jsonInput.value = JSON.stringify(playerSubmissionsData);
            if (proofInput) proofInput.value = playerSubmissionsData[0]?.proof_base64 || '';
        } else {
            if (!proofInput || !proofInput.value) {
                showToast("Veuillez joindre une capture d'écran faisant foi (Ctrl + V).", "error");
                return;
            }
            const jsonInput = document.getElementById('sub-player-submissions-json');
            if (jsonInput) jsonInput.value = '';
        }

        const confGame = document.getElementById('confirm-game');
        const confPlacement = document.getElementById('confirm-placement');
        const confSecRow = document.getElementById('confirm-sec-row');
        const confSecLabel = document.getElementById('confirm-sec-label');
        const confSecValue = document.getElementById('confirm-sec-value');
        const confPlayersRow = document.getElementById('confirm-players-row');
        const confPlayersList = document.getElementById('confirm-players-list');
        const confProofStatus = document.getElementById('confirm-proof-status');

        if (confGame) confGame.textContent = `${selectedMatch.game_name} — ${selectedMatch.title}`;
        if (confPlacement) {
            if (selectedMatch?.isScoreRanked) {
                confPlacement.textContent = "Calculé au score du round";
            } else if (selectedMatch?.noPlacementNeeded) {
                confPlacement.textContent = "Score cumulé direct";
            } else if (selectedMatch?.isFallGuys) {
                confPlacement.textContent = placementInput.value === '1' ? 'Show Remporté (+3 pts)' : 'Pas de victoire finale (+0 pt)';
            } else if (selectedMatch?.maxPlacement === 2) {
                confPlacement.textContent = placementInput.value === '1' ? 'Victoire' : 'Défaite';
            } else {
                confPlacement.textContent = `${placementInput.value}${placementInput.value === '1' ? 'er (Victoire / Top 1)' : 'e place'}`;
            }
        }

        const secInput = document.getElementById('sub-secondary');
        const secLabelEl = document.getElementById('secondary-stat-label');
        if (secInput && secLabelEl && secInput.value && secInput.offsetParent !== null) {
            if (confSecRow) confSecRow.style.display = 'block';
            if (confSecLabel) confSecLabel.textContent = secLabelEl.textContent;
            if (confSecValue) confSecValue.textContent = secInput.value;
        } else if (confSecRow) {
            confSecRow.style.display = 'none';
        }

        if (currentTeamPlayerCount > 1) {
            if (confPlayersRow) confPlayersRow.style.display = 'block';
            if (confPlayersList) {
                confPlayersList.innerHTML = playerSubmissionsData.map(p => `
                    <div style="display: flex; justify-content: space-between; padding: 0.15rem 0;">
                        <span>Joueur ${p.player_number} : <strong>${p.score}</strong></span>
                        <span style="color: var(--green, #22c55e);"><span class="papicon icon-check" aria-hidden="true"></span> Photo</span>
                    </div>
                `).join('');
            }
            if (confProofStatus) confProofStatus.textContent = `${currentTeamPlayerCount} captures prêtes ✓`;
        } else {
            if (confPlayersRow) confPlayersRow.style.display = 'none';
            if (confProofStatus) confProofStatus.textContent = 'Capture compressée prête ✓';
        }

        openModal('modal-confirm-submission');
    });

    if (finalConfirmBtn) {
        finalConfirmBtn.addEventListener('click', () => {
            isConfirmed = true;
            closeModal('modal-confirm-submission');
            submitBtn.disabled = true;
            submitBtn.textContent = "Transmission en cours...";
            form.submit();
        });
    }
}

function isFormDirty() {
    const proofInput = document.getElementById('sub-proof-base64');
    const placementInput = document.getElementById('sub-placement');
    const secInput = document.getElementById('sub-secondary');
    const hasProof = Boolean(proofInput && proofInput.value && proofInput.value.trim().length > 0);
    const hasPlacement = Boolean(placementInput && !selectedMatch?.noPlacementNeeded && !selectedMatch?.isFallGuys && placementInput.value.trim().length > 0);
    const hasSec = Boolean(secInput && secInput.value.trim().length > 0);
    const hasPlayersData = playerSubmissionsData.some(p => p.score > 0 || Boolean(p.proof_base64));
    return hasProof || hasPlacement || hasSec || hasPlayersData;
}

async function handleCaptainMatchUpdate() {
    try {
        const res = await fetch('/api/captain/matches', { credentials: 'same-origin' });
        if (!res.ok) return;
        const matches = await res.json();
        if (!Array.isArray(matches)) return;

        const knownIds = new Set(
            Array.from(document.querySelectorAll('#matches-list .match-item'))
                .map(el => el.getAttribute('data-match-id'))
                .filter(Boolean)
        );
        const newMatches = matches.filter(m => m && m.id && !knownIds.has(m.id));
        const listChanged = newMatches.length > 0 || matches.length !== knownIds.size;

        if (!listChanged) return;

        if (newMatches.length > 0) {
            const label = newMatches.length === 1
                ? `Nouveau match disponible : ${newMatches[0].title} (${newMatches[0].game_name})`
                : `${newMatches.length} nouveaux matchs disponibles pour votre équipe !`;

            if (isFormDirty()) {
                showToast(`${label} — actualisez après votre saisie.`, 'info');
                return;
            }
            showToast(label, 'info');
            setTimeout(() => window.location.reload(), 1200);
        } else if (!isFormDirty()) {
            window.location.reload();
        }
    } catch (e) {
        console.warn('Erreur lors de la vérification des nouveaux matchs:', e);
    }
}

async function updateCaptainPointsSilently() {
    try {
        const res = await fetch(window.location.href);
        if (!res.ok) return;
        const html = await res.text();
        const doc = new DOMParser().parseFromString(html, 'text/html');
        const newPts = doc.getElementById('captain-team-points');
        const currentPts = document.getElementById('captain-team-points');
        if (newPts && currentPts && newPts.textContent !== currentPts.textContent) {
            currentPts.textContent = newPts.textContent;
            showToast(`Points de votre équipe actualisés : ${newPts.textContent}`, 'info');
        }
    } catch (e) {
        console.warn('Erreur lors de la mise à jour silencieuse des points:', e);
    }
}

function initTournamentCountdown() {
    const kicker = document.getElementById('tournament-kicker');
    const textEl = document.getElementById('tournament-kicker-text');
    const dotEl = document.getElementById('tournament-kicker-dot');
    if (!kicker || !textEl) return;

    const startStr = kicker.dataset.start;
    const endStr = kicker.dataset.end;
    const isPaused = kicker.dataset.paused === 'true';
    const suffix = kicker.dataset.suffix || '';

    const setDotStatus = (color, glow = true, pulse = true) => {
        if (!dotEl) return;
        dotEl.style.backgroundColor = color;
        dotEl.style.boxShadow = glow ? `0 0 8px ${color}` : 'none';
        dotEl.style.animation = pulse ? 'pulse 2s infinite' : 'none';
    };

    if (isPaused) {
        textEl.textContent = `TOURNOI EN PAUSE${suffix}`;
        setDotStatus('#f97316');
        return;
    }

    const startDate = startStr ? new Date(startStr) : null;
    const endDate = endStr ? new Date(endStr) : null;

    if (!startDate && !endDate) return;

    function formatDiff(ms) {
        if (ms <= 0) return '00H 00M 00S';
        const totalSec = Math.floor(ms / 1000);
        const hours = Math.floor(totalSec / 3600);
        const minutes = Math.floor((totalSec % 3600) / 60);
        const seconds = totalSec % 60;
        const pad = (n) => String(n).padStart(2, '0');
        return `${pad(hours)}H ${pad(minutes)}M ${pad(seconds)}S`;
    }

    function update() {
        const now = new Date();

        if (startDate && now < startDate) {
            const diff = startDate.getTime() - now.getTime();
            textEl.textContent = `LANCEMENT DANS ${formatDiff(diff)}${suffix}`;
            setDotStatus('#eab308');
        } else if (endDate && now <= endDate) {
            const diff = endDate.getTime() - now.getTime();
            textEl.textContent = `TOURNOI EN COURS · RESTE ${formatDiff(diff)}${suffix}`;
            setDotStatus('var(--green-status, #10b981)');
        } else if (endDate && now > endDate) {
            textEl.textContent = `TOURNOI TERMINÉ · CLASSEMENT FINAL VALIDÉ`;
            setDotStatus('#64748b', false, false);
        } else {
            textEl.textContent = `TOURNOI EN COURS${suffix}`;
            setDotStatus('var(--green-status, #10b981)');
        }
    }

    update();
    setInterval(update, 1000);
}
