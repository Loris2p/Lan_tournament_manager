/**
 * Gestionnaire de Session & Garde de Navigation (auth-guard.js)
 * Interroge GET /api/auth/me et contrôle les accès selon le rôle.
 */

import { api } from './api.js';

/**
 * Normalise le rôle de l'utilisateur (chaîne simple ou objet enum serde)
 */
export function parseUserRole(role) {
    if (!role) return '';
    if (typeof role === 'string') return role.toLowerCase();
    if (typeof role === 'object') {
        const key = Object.keys(role)[0] || '';
        return key.toLowerCase();
    }
    return String(role).toLowerCase();
}

/**
 * Vérifie l'état d'authentification et redirige si non autorisé.
 * @param {'admin'|'captain'|'any'|null} requiredRole - Rôle requis pour la page (ou null pour page publique)
 * @returns {Promise<{username: string, role: any, normalizedRole: string}|null>}
 */
export async function checkAuth(requiredRole = null) {
    const isLoginPage = window.location.pathname.endsWith('/login.html') || window.location.pathname === '/login';

    try {
        const user = await api.get('/api/auth/me');
        // user = { sub: "...", username: "...", role: "ADMIN" | { "CAPTAIN": { ... } } }
        const role = parseUserRole(user.role);

        // Si l'utilisateur est sur la page login alors qu'il est déjà connecté :
        if (isLoginPage) {
            if (role === 'admin') {
                window.location.replace('/admin.html');
                return user;
            } else if (role === 'captain') {
                window.location.replace('/captain.html');
                return user;
            }
        }

        // Si la page requiert un rôle spécifique :
        if (requiredRole && requiredRole !== 'any') {
            if (role !== requiredRole.toLowerCase()) {
                // Mauvais rôle : redirection vers son espace légitime
                if (role === 'admin') {
                    window.location.replace('/admin.html');
                } else {
                    window.location.replace('/captain.html');
                }
                return null;
            }
        }

        // Met à jour le bouton de connexion en bouton espace si présent sur les pages publiques
        updateHeaderUser(user, role);
        return { ...user, normalizedRole: role };
    } catch (err) {
        // Utilisateur non connecté : rediriger uniquement si la page est protégée et qu'on n'est pas déjà sur login
        if (requiredRole && requiredRole !== 'any' && !isLoginPage) {
            window.location.replace('/login.html');
            return null;
        }
        return null;
    }
}

/**
 * Met à jour le bouton d'authentification du header avec le pseudo et lien vers son dashboard
 */
function updateHeaderUser(user, role) {
    const authBtn = document.getElementById('auth-btn');
    if (!authBtn || !user) return;

    const destination = role === 'admin' ? '/admin.html' : '/captain.html';
    authBtn.textContent = `${user.username.toUpperCase()} [${role.toUpperCase()}]`;
    authBtn.href = destination;
}

/**
 * Déconnexion centralisée
 */
export async function logout() {
    try {
        await api.post('/api/auth/logout');
    } catch (_) {
        // Ignorer l'erreur réseau éventuelle
    }
    window.location.replace('/login.html');
}
