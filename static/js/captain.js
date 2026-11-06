import { wsClient } from './ws.js';
import { showToast } from './ui.js';

let selectedMatch = null;

document.addEventListener('DOMContentLoaded', () => {
    initMatches();
    initDropzoneAndPaste();
    initFormSubmit();

    wsClient.connect();
    wsClient.on('LEADERBOARD_UPDATE', () => {
        window.location.reload();
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

    selectedMatch = { id: matchId, title, game_name: game, instructions };

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

    // Adaptation du placement
    const placementInput = document.getElementById('sub-placement');
    const placementHint = document.getElementById('placement-hint');
    if (placementInput) {
        placementInput.max = maxPlacement;
        if (format.toLowerCase() === 'duel') {
            placementInput.placeholder = "1 (Victoire) ou 2 (Défaite)";
            if (placementHint) placementHint.textContent = "1 = Victoire (+3 pts), 2 = Défaite (+1 pt)";
        } else {
            placementInput.placeholder = `Ex: 1 (Top 1), 2... (max ${maxPlacement})`;
            if (placementHint) placementHint.textContent = `Place finale obtenue dans le salon (1 à ${maxPlacement})`;
        }
    }

    // Adaptation de la statistique secondaire conditionnelle
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
}

function initDropzoneAndPaste() {
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const clearBtn = document.getElementById('clear-image-btn');

    if (!dropzone || !fileInput) return;

    window.addEventListener('paste', (e) => {
        if (!selectedMatch) return;
        const items = (e.clipboardData || e.originalEvent?.clipboardData)?.items;
        if (!items) return;

        for (const item of items) {
            if (item.type.includes('image')) {
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

    if (clearBtn) {
        clearBtn.addEventListener('click', resetImage);
    }
}

function resetImage() {
    const proofInput = document.getElementById('sub-proof-base64');
    const fileInput = document.getElementById('file-input');
    const previewWrapper = document.getElementById('preview-wrapper');
    const dropzonePrompt = document.getElementById('dropzone-prompt');

    if (proofInput) proofInput.value = '';
    if (fileInput) fileInput.value = '';
    if (previewWrapper) previewWrapper.style.display = 'none';
    if (dropzonePrompt) dropzonePrompt.style.display = 'block';
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
            const proofInput = document.getElementById('sub-proof-base64');
            if (proofInput) proofInput.value = dataUrl;

            const sizeKb = Math.round((dataUrl.length * 3 / 4) / 1024);
            const previewImg = document.getElementById('preview-img');
            const previewBadge = document.getElementById('preview-size-badge');

            if (previewImg) previewImg.src = dataUrl;
            if (previewBadge) previewBadge.textContent = `${sizeKb} Ko [Compressé]`;

            const prompt = document.getElementById('dropzone-prompt');
            const wrapper = document.getElementById('preview-wrapper');
            if (prompt) prompt.style.display = 'none';
            if (wrapper) wrapper.style.display = 'block';
        };
        img.src = event.target.result;
    };
    reader.readAsDataURL(file);
}

function initFormSubmit() {
    const form = document.getElementById('submission-form');
    const submitBtn = document.getElementById('submit-score-btn');
    if (!form || !submitBtn) return;

    form.addEventListener('submit', (e) => {
        if (!selectedMatch) {
            e.preventDefault();
            showToast("Veuillez sélectionner un match.", "error");
            return;
        }

        const proofInput = document.getElementById('sub-proof-base64');
        if (!proofInput || !proofInput.value) {
            e.preventDefault();
            showToast("Veuillez joindre une capture d'écran faisant foi (Ctrl + V).", "error");
            return;
        }

        submitBtn.disabled = true;
        submitBtn.textContent = "Transmission en cours...";
    });
}
