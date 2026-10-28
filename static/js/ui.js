/**
 * Composants d'interface réutilisables (ui.js)
 * Toasts de notification, modales et utilitaires d'affichage.
 */

// Conteneur de toasts créé à la volée
let toastContainer = null;

function getToastContainer() {
    if (!toastContainer) {
        toastContainer = document.createElement('div');
        toastContainer.className = 'toast-container';
        document.body.appendChild(toastContainer);
    }
    return toastContainer;
}

/**
 * Affiche un toast temporaire
 * @param {string} message - Texte de l'alerte
 * @param {'success'|'error'|'info'} type - Type visuel
 * @param {number} duration - Durée en ms (défaut 4000)
 */
export function showToast(message, type = 'info', duration = 4000) {
    const container = getToastContainer();
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;

    const icon = type === 'success' ? '✓' : (type === 'error' ? '✕' : 'ℹ');

    toast.innerHTML = `
        <span class="toast-icon">${icon}</span>
        <span class="toast-message">${escapeHtml(message)}</span>
    `;

    container.appendChild(toast);

    // Animation d'apparition
    requestAnimationFrame(() => {
        toast.classList.add('visible');
    });

    // Disparition automatique
    setTimeout(() => {
        toast.classList.remove('visible');
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 300);
    }, duration);
}

/**
 * Ouvre une modale par son ID
 * @param {string} modalId 
 */
export function openModal(modalId) {
    const modal = document.getElementById(modalId);
    if (!modal) return;
    modal.classList.add('active');
    document.body.style.overflow = 'hidden';

    // Fermeture avec Échap
    const handleKeydown = (e) => {
        if (e.key === 'Escape') {
            closeModal(modalId);
            window.removeEventListener('keydown', handleKeydown);
        }
    };
    window.addEventListener('keydown', handleKeydown);
}

/**
 * Ferme une modale par son ID
 * @param {string} modalId 
 */
export function closeModal(modalId) {
    const modal = document.getElementById(modalId);
    if (!modal) return;
    modal.classList.remove('active');
    document.body.style.overflow = '';
}

/**
 * Initialise la fermeture des modales sur les boutons data-close-modal
 */
export function initModalListeners() {
    document.addEventListener('click', (e) => {
        const closeBtn = e.target.closest('[data-close-modal]');
        if (closeBtn) {
            const modalId = closeBtn.getAttribute('data-close-modal');
            closeModal(modalId);
        }
        // Clic sur l'overlay pour fermer
        if (e.target.classList.contains('modal-overlay')) {
            e.target.classList.remove('active');
            document.body.style.overflow = '';
        }
    });
}

export function escapeHtml(str) {
    if (!str && str !== 0) return '';
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
