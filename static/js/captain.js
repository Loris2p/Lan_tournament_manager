/**
 * Logique de l'Espace Capitaine (captain.js)
 * - Récupération des matchs ouverts assignés (/api/captain/matches)
 * - Formulaire dynamique selon les règles du jeu
 * - Support complet du collage presse-papier (Ctrl + V)
 * - Compression Canvas d'image en 1080p JPEG côté client
 * - Envoi JSON du score et de l'image base64 (/api/captain/submissions)
 */

import { api } from './api.js';
import { checkAuth, logout } from './auth-guard.js';
import { wsClient } from './ws.js';
import { showToast, escapeHtml } from './ui.js';

let currentUser = null;
let openMatches = [];
let selectedMatch = null;
let compressedBase64 = null;

document.addEventListener('DOMContentLoaded', async () => {
    currentUser = await checkAuth('captain');
    if (!currentUser) return;

    initCaptainHeader();
    initMatches();
    initDropzoneAndPaste();
    initFormSubmit();

    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', () => {
        initCaptainHeader();
        initMatches();
    });

    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) logoutBtn.addEventListener('click', logout);

    const newSubBtn = document.getElementById('new-submission-btn');
    if (newSubBtn) {
        newSubBtn.addEventListener('click', () => {
            document.getElementById('submission-success').style.display = 'none';
            document.getElementById('submission-form').style.display = 'block';
            resetImage();
        });
    }
});

function initCaptainHeader() {
    const badge = document.getElementById('captain-team-badge');
    const title = document.getElementById('captain-team-name');
    const pointsEl = document.getElementById('captain-team-points');

    if (badge) badge.textContent = `ÉQUIPE : ${currentUser.username.toUpperCase()}`;
    if (title) title.textContent = currentUser.username;

    api.get('/api/leaderboard').then(teams => {
        const team = teams.find(t => 
            (t.captain_username && t.captain_username.toLowerCase() === currentUser.username.toLowerCase()) ||
            (t.name && t.name.toLowerCase() === currentUser.username.toLowerCase())
        );
        if (team) {
            if (badge) badge.textContent = `ÉQUIPE : ${team.name.toUpperCase()}`;
            if (title) title.textContent = team.name;
            if (pointsEl) pointsEl.textContent = `${team.total_points ?? team.points ?? 0} PTS`;
        }
    }).catch(() => {});
}

async function initMatches() {
    const listEl = document.getElementById('matches-list');
    const countBadge = document.getElementById('open-matches-count');

    try {
        openMatches = await api.get('/api/captain/matches');

        if (countBadge) {
            countBadge.textContent = `${openMatches.length} ACTIF${openMatches.length > 1 ? 'S' : ''}`;
        }

        if (!openMatches || openMatches.length === 0) {
            listEl.innerHTML = `<div class="empty-state">Aucun match ouvert assigné à votre équipe pour le moment. Attendez l'ouverture d'un round par les arbitres.</div>`;
            return;
        }

        listEl.innerHTML = openMatches.map((m, idx) => `
            <div class="match-item ${idx === 0 ? 'selected' : ''}" data-match-id="${m.id}">
                <div class="match-item-header">
                    <span class="badge-format">${m.title || (m.format === 'duel' ? 'DUEL' : 'BRAWL')}</span>
                    <strong>${escapeHtml(m.game_name || 'Match')}</strong>
                </div>
                <p class="match-item-sub">${escapeHtml(m.instructions || 'Aucune consigne spécifique')}</p>
            </div>
        `).join('');

        const items = listEl.querySelectorAll('.match-item');
        items.forEach(item => {
            item.addEventListener('click', () => {
                items.forEach(i => i.classList.remove('selected'));
                item.classList.add('selected');
                const matchId = item.getAttribute('data-match-id');
                selectMatch(matchId);
            });
        });

        if (openMatches.length > 0) {
            selectMatch(openMatches[0].id);
        }
    } catch (err) {
        listEl.innerHTML = `<div class="empty-state" style="color: var(--red);">Erreur : ${escapeHtml(err.message)}</div>`;
    }
}

function selectMatch(matchId) {
    selectedMatch = openMatches.find(m => m.id === matchId);
    if (!selectedMatch) return;

    const detailsPanel = document.getElementById('selected-match-details');
    const noMatchAlert = document.getElementById('no-match-alert');
    const form = document.getElementById('submission-form');
    const subMatchId = document.getElementById('sub-match-id');

    detailsPanel.style.display = 'block';
    noMatchAlert.style.display = 'none';
    form.style.display = 'block';
    subMatchId.value = selectedMatch.id;

    document.getElementById('selected-match-game').textContent = selectedMatch.game_name || 'Match';
    document.getElementById('selected-match-format').textContent = selectedMatch.format === 'duel' ? 'FORMAT : DUEL 1V1' : 'FORMAT : BRAWL MULTI-ÉQUIPES';
    document.getElementById('selected-match-instructions').textContent = selectedMatch.instructions || 'Aucune consigne spécifique pour ce lobby.';
    document.getElementById('selected-match-rules').textContent = selectedMatch.rules || 'Règles standards du catalogue.';

    const placementInput = document.getElementById('sub-placement');
    const placementHint = document.getElementById('placement-hint');
    if (selectedMatch.max_placement) {
        placementInput.max = selectedMatch.max_placement;
        placementHint.textContent = `Classement compris entre 1 et ${selectedMatch.max_placement}`;
    }

    const secGroup = document.getElementById('secondary-stat-group');
    const secLabel = document.getElementById('secondary-stat-label');
    const secInput = document.getElementById('sub-secondary');
    const secHint = document.getElementById('secondary-stat-hint');

    if (selectedMatch.secondary_stat_label && selectedMatch.secondary_stat_label.trim().length > 0) {
        secGroup.style.display = 'block';
        secLabel.textContent = selectedMatch.secondary_stat_label.toUpperCase();
        secInput.max = selectedMatch.max_secondary_stat || 9999;
        secHint.textContent = `Points supplémentaires par unité (Max ${selectedMatch.max_secondary_stat || 'illimité'})`;
    } else {
        secGroup.style.display = 'none';
        secInput.value = '0';
    }
}

/* ==========================================================================
   MODULE CANVAS : COMPRESSION D'IMAGE & SUPPORT CTRL + V
   ========================================================================== */

function initDropzoneAndPaste() {
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const clearBtn = document.getElementById('clear-image-btn');

    window.addEventListener('paste', (e) => {
        if (!selectedMatch) return;
        const items = (e.clipboardData || e.originalEvent.clipboardData).items;
        for (const item of items) {
            if (item.type.indexOf('image') !== -1) {
                const file = item.getAsFile();
                processAndCompressImage(file);
                showToast("Capture d'écran collée depuis le presse-papier !", "info");
                break;
            }
        }
    });

    fileInput.addEventListener('change', (e) => {
        if (e.target.files && e.target.files[0]) {
            processAndCompressImage(e.target.files[0]);
        }
    });

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

    clearBtn.addEventListener('click', resetImage);
}

function resetImage() {
    compressedBase64 = null;
    document.getElementById('file-input').value = '';
    document.getElementById('preview-wrapper').style.display = 'none';
    document.getElementById('dropzone-prompt').style.display = 'block';
}

function processAndCompressImage(file) {
    if (!file || !file.type.startsWith('image/')) {
        showToast("Le fichier sélectionné n'est pas une image valide.", "error");
        return;
    }

    const reader = new FileReader();
    reader.onload = (event) => {
        const img = new Image();
        img.onload = () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');

            const MAX_WIDTH = 1920;
            const MAX_HEIGHT = 1080;
            let width = img.width;
            let height = img.height;

            if (width > height) {
                if (width > MAX_WIDTH) {
                    height *= MAX_WIDTH / width;
                    width = MAX_WIDTH;
                }
            } else {
                if (height > MAX_HEIGHT) {
                    width *= MAX_HEIGHT / height;
                    height = MAX_HEIGHT;
                }
            }

            canvas.width = width;
            canvas.height = height;
            ctx.drawImage(img, 0, 0, width, height);

            // Exportation en Base64 data:image/jpeg
            const dataUrl = canvas.toDataURL('image/jpeg', 0.80);
            compressedBase64 = dataUrl;

            const sizeKb = Math.round((dataUrl.length * 3 / 4) / 1024);

            const previewImg = document.getElementById('preview-img');
            const previewBadge = document.getElementById('preview-size-badge');
            previewImg.src = dataUrl;
            previewBadge.textContent = `${sizeKb} Ko [Compressé]`;

            document.getElementById('dropzone-prompt').style.display = 'none';
            document.getElementById('preview-wrapper').style.display = 'block';
        };
        img.src = event.target.result;
    };
    reader.readAsDataURL(file);
}

function initFormSubmit() {
    const form = document.getElementById('submission-form');
    const submitBtn = document.getElementById('submit-score-btn');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();

        if (!selectedMatch) {
            showToast("Veuillez sélectionner un match.", "error");
            return;
        }

        if (!compressedBase64) {
            showToast("Veuillez joindre une capture d'écran faisant foi (Ctrl + V).", "error");
            return;
        }

        const placement = parseInt(document.getElementById('sub-placement').value, 10);
        const secondary = parseInt(document.getElementById('sub-secondary').value, 10) || 0;

        if (selectedMatch.max_placement && placement > selectedMatch.max_placement) {
            showToast(`Le classement ne peut pas dépasser ${selectedMatch.max_placement} pour ce jeu.`, "error");
            return;
        }
        if (selectedMatch.max_secondary_stat && secondary > selectedMatch.max_secondary_stat) {
            showToast(`La statistique secondaire ne peut pas dépasser ${selectedMatch.max_secondary_stat}.`, "error");
            return;
        }

        submitBtn.disabled = true;
        submitBtn.textContent = "Transmission en cours...";

        try {
            await api.post('/api/captain/submissions', {
                match_id: selectedMatch.id,
                score: {
                    placement,
                    secondary_stat: secondary
                },
                proof_base64: compressedBase64
            });

            showToast("Score transmis avec succès aux arbitres !", "success");

            form.style.display = 'none';
            document.getElementById('submission-success').style.display = 'block';
        } catch (err) {
            showToast(err.message, "error");
        } finally {
            submitBtn.disabled = false;
            submitBtn.textContent = "Envoyer le Score pour Arbitrage";
        }
    });
}
