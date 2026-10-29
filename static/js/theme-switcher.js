/**
 * Gestionnaire d'ambiances visuelles et de dispositions de page (theme-switcher.js)
 * Permet d'alterner dynamiquement entre :
 * - Thèmes de couleurs : Pro Esport Glass, Cyber Crimson, Tactical HUD
 * - Dispositions / Layouts : Split-Screen Broadcast, Bento Grid, Sidebar HUD, Stack Classique
 */

const THEMES = [
    { id: 'modern-esport', label: 'Pro Esport Glass' },
    { id: 'cyber-crimson', label: 'Cyber Crimson' },
    { id: 'tactical-hud',  label: 'Tactical HUD' }
];

const LAYOUTS = [
    { id: 'split-screen',  label: 'Split Broadcast' },
    { id: 'bento-grid',    label: 'Bento Grid' },
    { id: 'sidebar-hud',   label: 'Sidebar Cockpit' },
    { id: 'classic-stack', label: 'Stack Classique' }
];

const THEME_KEY = 'ltm_theme';
const LAYOUT_KEY = 'ltm_layout';

export function getActiveTheme() {
    return localStorage.getItem(THEME_KEY) || 'modern-esport';
}

export function getActiveLayout() {
    return localStorage.getItem(LAYOUT_KEY) || 'split-screen';
}

export function setTheme(themeId) {
    if (!THEMES.some(t => t.id === themeId)) {
        themeId = 'modern-esport';
    }
    document.documentElement.setAttribute('data-theme', themeId);
    localStorage.setItem(THEME_KEY, themeId);

    const selector = document.getElementById('theme-select');
    if (selector && selector.value !== themeId) {
        selector.value = themeId;
    }
}

export function setLayout(layoutId) {
    if (!LAYOUTS.some(l => l.id === layoutId)) {
        layoutId = 'split-screen';
    }
    document.documentElement.setAttribute('data-layout', layoutId);
    localStorage.setItem(LAYOUT_KEY, layoutId);

    const selector = document.getElementById('layout-select');
    if (selector && selector.value !== layoutId) {
        selector.value = layoutId;
    }
}

export function initSwitcher() {
    // 1. Appliquer thème et disposition actifs
    setTheme(getActiveTheme());
    setLayout(getActiveLayout());

    // 2. Insérer les sélecteurs dans la topbar si présente
    const topbarRight = document.querySelector('.topbar-right');
    if (!topbarRight || document.getElementById('switchers-container')) {
        return;
    }

    const container = document.createElement('div');
    container.id = 'switchers-container';
    container.className = 'switchers-wrapper';

    // Sélecteur de Layout / Disposition
    const layoutSelect = document.createElement('select');
    layoutSelect.id = 'layout-select';
    layoutSelect.className = 'switcher-select-input';
    layoutSelect.title = 'Changer la disposition de la page';

    const activeLayout = getActiveLayout();
    LAYOUTS.forEach(l => {
        const opt = document.createElement('option');
        opt.value = l.id;
        opt.textContent = `📐 ${l.label}`;
        if (l.id === activeLayout) opt.selected = true;
        layoutSelect.appendChild(opt);
    });

    layoutSelect.addEventListener('change', (e) => {
        setLayout(e.target.value);
    });

    // Sélecteur de Palette / Style
    const themeSelect = document.createElement('select');
    themeSelect.id = 'theme-select';
    themeSelect.className = 'switcher-select-input';
    themeSelect.title = 'Changer l\'ambiance visuelle du tournoi';

    const activeTheme = getActiveTheme();
    THEMES.forEach(t => {
        const opt = document.createElement('option');
        opt.value = t.id;
        opt.textContent = `🎨 ${t.label}`;
        if (t.id === activeTheme) opt.selected = true;
        themeSelect.appendChild(opt);
    });

    themeSelect.addEventListener('change', (e) => {
        setTheme(e.target.value);
    });

    container.appendChild(layoutSelect);
    container.appendChild(themeSelect);

    // Insérer juste avant l'indicateur WebSocket
    const wsIndicator = document.getElementById('ws-indicator');
    if (wsIndicator) {
        topbarRight.insertBefore(container, wsIndicator);
    } else {
        topbarRight.prepend(container);
    }
}

// Auto-initialisation au chargement du DOM
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSwitcher);
} else {
    initSwitcher();
}
