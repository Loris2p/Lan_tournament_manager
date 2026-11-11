// Script synchrone chargé dans <head> : applique thème + disposition avant le premier rendu (évite le flash).
// Les listes doivent rester alignées avec THEMES / LAYOUTS de theme-switcher.js.
(function () {
    try {
        const params = new URLSearchParams(window.location.search);
        const qTheme = params.get('theme') || params.get('layout');
        if (qTheme === 'tactical-pro' || qTheme === 'tactical-deck') {
            localStorage.setItem('ltm_layout', 'tactical-deck');
            localStorage.setItem('ltm_theme', 'tactical-broadcast');
        } else if (qTheme === 'arena') {
            localStorage.setItem('ltm_layout', 'arena');
            localStorage.setItem('ltm_theme', 'tactical-broadcast');
        }

        const themes = ['tactical-broadcast', 'neo-arcade', 'industrial-brutalism', 'stealth-laser'];
        let t = localStorage.getItem('ltm_theme');
        if (themes.indexOf(t) === -1) t = 'tactical-broadcast';
        const l = 'arena';
        localStorage.setItem('ltm_layout', 'arena');
        const root = document.documentElement;
        root.setAttribute('data-theme', t);
        root.setAttribute('data-layout', l);
        const link = document.getElementById('ltm-theme-style');
        if (link) link.href = '/css/themes/' + t + '.css';

        // Le serveur rend un HTML différent pour Arena d'après le cookie ltm_layout.
        // Si le cookie diverge du choix mémorisé (première visite, cookie effacé), on le resynchronise
        // et on recharge une seule fois par valeur (garde-fou anti-boucle si cookies bloqués).
        const m = document.cookie.match(/(?:^|; )ltm_layout=([^;]*)/);
        const c = m ? m[1] : null;
        if ((c === 'arena') !== (l === 'arena')) {
            document.cookie = 'ltm_layout=' + l + '; path=/; max-age=31536000; SameSite=Lax';
            if (sessionStorage.getItem('ltm_sync') !== l) {
                sessionStorage.setItem('ltm_sync', l);
                location.reload();
            }
        }
    } catch (e) {}
})();
