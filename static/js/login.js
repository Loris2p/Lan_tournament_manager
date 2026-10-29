/**
 * Logique d'authentification (login.js)
 * Gère l'assistant de premier démarrage (/api/auth/setup)
 * et la connexion classique (/api/auth/login) avec redirection par rôle.
 */

import { api } from './api.js';
import { checkAuth } from './auth-guard.js';
import { wsClient } from './ws.js';
import { showToast } from './ui.js';

document.addEventListener('DOMContentLoaded', async () => {
    // Si déjà connecté, checkAuth redirige vers sa page respective. Sinon renvoie null sans boucle
    await checkAuth(null);
    wsClient.connect();
    initAuthPage();
});

async function initAuthPage() {
    const loadingEl = document.getElementById('auth-loading');
    const setupWrapper = document.getElementById('setup-form-wrapper');
    const loginWrapper = document.getElementById('login-form-wrapper');

    try {
        const status = await api.get('/api/auth/setup-status');
        // Le backend renvoie { "needs_setup": boolean }
        const needsSetup = status.needs_setup ?? status.setup_required ?? false;

        loadingEl.style.display = 'none';

        if (needsSetup) {
            setupWrapper.style.display = 'block';
            loginWrapper.style.display = 'none';
            setupSetupForm();
        } else {
            loginWrapper.style.display = 'block';
            setupWrapper.style.display = 'none';
            setupLoginForm();
        }
    } catch (err) {
        loadingEl.innerHTML = `<span style="color: var(--red);">Erreur de communication : ${err.message}</span>`;
    }
}

function showAlert(message, isError = true) {
    const alertEl = document.getElementById('auth-alert');
    if (!alertEl) return;
    alertEl.textContent = message;
    alertEl.className = `auth-alert ${isError ? 'auth-alert-error' : 'auth-alert-success'}`;
    alertEl.style.display = 'block';
}

function clearAlert() {
    const alertEl = document.getElementById('auth-alert');
    if (alertEl) alertEl.style.display = 'none';
}

function setupSetupForm() {
    const form = document.getElementById('setup-form');
    const submitBtn = document.getElementById('setup-submit-btn');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        clearAlert();

        const username = document.getElementById('setup-username').value.trim();
        const password = document.getElementById('setup-password').value;
        const confirm = document.getElementById('setup-password-confirm').value;

        if (password !== confirm) {
            showAlert("Les mots de passe ne correspondent pas.");
            return;
        }

        if (password.length < 4) {
            showAlert("Le mot de passe doit contenir au moins 4 caractères.");
            return;
        }

        submitBtn.disabled = true;
        submitBtn.textContent = "Création en cours...";

        try {
            await api.post('/api/auth/setup', { username, password });
            showToast("Administrateur créé avec succès ! Verrouillage du serveur...", "success");
            setTimeout(() => {
                window.location.replace('/admin.html');
            }, 800);
        } catch (err) {
            showAlert(err.message);
            submitBtn.disabled = false;
            submitBtn.textContent = "Créer l'Administrateur & Démarrer";
        }
    });
}

function setupLoginForm() {
    const form = document.getElementById('login-form');
    const submitBtn = document.getElementById('login-submit-btn');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        clearAlert();

        const username = document.getElementById('login-username').value.trim();
        const password = document.getElementById('login-password').value;

        submitBtn.disabled = true;
        submitBtn.textContent = "Vérification...";

        try {
            const res = await api.post('/api/auth/login', { username, password });
            // res = { role: "ADMIN" | "CAPTAIN", redirect: "..." }
            const role = (res.role || '').toLowerCase();

            showToast("Connexion réussie !", "success");

            setTimeout(() => {
                if (res.redirect) {
                    window.location.replace(res.redirect);
                } else if (role.includes('admin')) {
                    window.location.replace('/admin.html');
                } else {
                    window.location.replace('/captain.html');
                }
            }, 400);
        } catch (err) {
            showAlert(err.message);
            submitBtn.disabled = false;
            submitBtn.textContent = "Se Connecter";
        }
    });
}
