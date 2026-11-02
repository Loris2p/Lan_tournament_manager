/**
 * Bannière d'information Cookies (cookie-banner.js)
 * Conforme RGPD / ePrivacy : Information claire sur l'utilisation du cookie technique de session.
 */

const STORAGE_KEY = 'ltm_cookie_ack';

export function initCookieBanner() {
    if (localStorage.getItem(STORAGE_KEY)) return;

    const banner = document.createElement('aside');
    banner.className = 'cookie-banner';
    banner.setAttribute('role', 'alert');
    banner.setAttribute('aria-label', 'Information sur les cookies');
    banner.innerHTML = `
        <div class="cookie-banner-content">
            <span class="cookie-banner-icon" aria-hidden="true">🍪</span>
            <p>
                Ce site utilise uniquement un <strong>cookie de session technique</strong> (<code>lan_session</code>) 
                strictement nécessaire à votre authentification et au bon déroulement du tournoi. 
                Aucun traceur publicitaire ou d'analyse n'est utilisé.
            </p>
        </div>
        <div class="cookie-banner-actions">
            <button id="btn-accept-cookie" class="btn btn-sm btn-primary">J'ai compris</button>
        </div>
    `;

    document.body.appendChild(banner);

    // Animation d'apparition
    requestAnimationFrame(() => {
        banner.classList.add('visible');
    });

    const acceptBtn = banner.querySelector('#btn-accept-cookie');
    if (acceptBtn) {
        acceptBtn.addEventListener('click', () => {
            localStorage.setItem(STORAGE_KEY, 'true');
            banner.classList.remove('visible');
            setTimeout(() => banner.remove(), 300);
        });
    }
}

// Exécution automatique dès que le DOM est prêt
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initCookieBanner);
} else {
    initCookieBanner();
}
