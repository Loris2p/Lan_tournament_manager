// Script synchrone chargé dans <head> : applique thème + disposition avant le premier rendu (évite le flash).
// Les listes doivent rester alignées avec THEMES / LAYOUTS de theme-switcher.js.
(function () {
    try {
        var themes = ['tactical-broadcast', 'neo-arcade', 'industrial-brutalism', 'stealth-laser'];
        var layouts = ['tactical-deck', 'bento-grid'];
        var t = localStorage.getItem('ltm_theme');
        if (themes.indexOf(t) === -1) t = 'tactical-broadcast';
        var l = localStorage.getItem('ltm_layout');
        if (layouts.indexOf(l) === -1) l = 'tactical-deck';
        var root = document.documentElement;
        root.setAttribute('data-theme', t);
        root.setAttribute('data-layout', l);
        var link = document.getElementById('ltm-theme-style');
        if (link) link.href = '/css/themes/' + t + '.css';
    } catch (e) {}
})();
