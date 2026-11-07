// Script synchrone chargé dans <head> : applique thème + disposition avant le premier rendu (évite le flash).
// Les listes doivent rester alignées avec THEMES / LAYOUTS de theme-switcher.js.
(function () {
    try {
        var params = new URLSearchParams(window.location.search);
        var qTheme = params.get('theme') || params.get('layout');
        if (qTheme === 'tactical-pro' || qTheme === 'tactical-deck') {
            localStorage.setItem('ltm_layout', 'tactical-deck');
            localStorage.setItem('ltm_theme', 'tactical-broadcast');
        } else if (qTheme === 'arena') {
            localStorage.setItem('ltm_layout', 'arena');
            localStorage.setItem('ltm_theme', 'tactical-broadcast');
        }

        var themes = ['tactical-broadcast', 'neo-arcade', 'industrial-brutalism', 'stealth-laser'];
        var layouts = ['arena', 'tactical-deck', 'bento-grid'];
        var t = localStorage.getItem('ltm_theme');
        if (themes.indexOf(t) === -1) t = 'tactical-broadcast';
        var l = localStorage.getItem('ltm_layout');
        if (layouts.indexOf(l) === -1) l = 'arena';
        var root = document.documentElement;
        root.setAttribute('data-theme', t);
        root.setAttribute('data-layout', l);
        var link = document.getElementById('ltm-theme-style');
        if (link) link.href = '/css/themes/' + t + '.css';

        // Le serveur rend un HTML différent pour Arena d'après le cookie ltm_layout.
        // Si le cookie diverge du choix mémorisé (première visite, cookie effacé), on le resynchronise
        // et on recharge une seule fois par valeur (garde-fou anti-boucle si cookies bloqués).
        var m = document.cookie.match(/(?:^|; )ltm_layout=([^;]*)/);
        var c = m ? m[1] : null;
        if ((c === 'arena') !== (l === 'arena')) {
            document.cookie = 'ltm_layout=' + l + '; path=/; max-age=31536000; SameSite=Lax';
            if (sessionStorage.getItem('ltm_sync') !== l) {
                sessionStorage.setItem('ltm_sync', l);
                location.reload();
            }
        }
    } catch (e) {}
})();
