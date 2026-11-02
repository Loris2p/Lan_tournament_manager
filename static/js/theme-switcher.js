/**
 * Gestionnaire d'ambiances visuelles et de dispositions de page (theme-switcher.js)
 * Permet d'alterner dynamiquement entre :
 * - Thèmes de couleurs : Pro Esport Glass, Cyber Crimson, Tactical HUD
 * - Dispositions / Layouts : Split-Screen Broadcast, Bento Grid, Sidebar HUD, Stack Classique
 */

const THEMES = [
    { id: 'modern-esport', label: 'Glass' },
    { id: 'cyber-crimson', label: 'Crimson' }
];

const LAYOUTS = [
    { id: 'split-screen',  label: 'Split' },
    { id: 'sidebar-hud',   label: 'Sidebar' },
    { id: 'classic-stack', label: 'Stack' }
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

    // 2. Insérer le menu de paramètres dans la topbar si présente
    const topbarRight = document.querySelector('.topbar-right');
    if (!topbarRight || document.getElementById('settings-menu-container')) {
        return;
    }

    // Conteneur principal
    const container = document.createElement('div');
    container.id = 'settings-menu-container';
    container.className = 'settings-menu-wrapper';

    // Bouton déclencheur
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.id = 'settings-toggle-btn';
    btn.className = 'settings-trigger-btn';
    btn.setAttribute('aria-haspopup', 'true');
    btn.setAttribute('aria-expanded', 'false');
    btn.title = 'Paramètres d\'affichage';
    btn.innerHTML = `<span class="papicon icon-gears" aria-hidden="true"></span><span class="settings-btn-label">Options</span>`;

    // Panneau déroulant (masqué par défaut)
    const dropdown = document.createElement('div');
    dropdown.id = 'settings-dropdown';
    dropdown.className = 'settings-dropdown';
    dropdown.setAttribute('hidden', '');

    // En-tête
    const header = document.createElement('div');
    header.className = 'settings-dropdown-header';
    header.innerHTML = `<span class="settings-dropdown-title"><span class="papicon icon-gears" aria-hidden="true"></span> Paramètres</span>`;
    dropdown.appendChild(header);

    // Corps du menu
    const body = document.createElement('div');
    body.className = 'settings-dropdown-body';

    // Groupe 1 : Ambiance visuelle (Thème)
    const themeGroup = document.createElement('div');
    themeGroup.className = 'settings-group';

    const themeLabel = document.createElement('label');
    themeLabel.htmlFor = 'theme-select';
    themeLabel.className = 'settings-label';
    themeLabel.innerHTML = `<span class="papicon icon-palette" aria-hidden="true"></span><span>Ambiance visuelle</span>`;

    const themeSelect = document.createElement('select');
    themeSelect.id = 'theme-select';
    themeSelect.className = 'switcher-select-input';
    themeSelect.title = 'Changer l\'ambiance visuelle du tournoi';

    const activeTheme = getActiveTheme();
    THEMES.forEach(t => {
        const opt = document.createElement('option');
        opt.value = t.id;
        opt.textContent = t.label;
        if (t.id === activeTheme) opt.selected = true;
        themeSelect.appendChild(opt);
    });

    themeSelect.addEventListener('change', (e) => {
        setTheme(e.target.value);
    });

    themeGroup.appendChild(themeLabel);
    themeGroup.appendChild(themeSelect);
    body.appendChild(themeGroup);

    // Groupe 2 : Disposition (Layout)
    const layoutGroup = document.createElement('div');
    layoutGroup.className = 'settings-group';

    const layoutLabel = document.createElement('label');
    layoutLabel.htmlFor = 'layout-select';
    layoutLabel.className = 'settings-label';
    layoutLabel.innerHTML = `<span class="papicon icon-grid" aria-hidden="true"></span><span>Disposition</span>`;

    const layoutSelect = document.createElement('select');
    layoutSelect.id = 'layout-select';
    layoutSelect.className = 'switcher-select-input';
    layoutSelect.title = 'Changer la disposition de la page';

    const activeLayout = getActiveLayout();
    LAYOUTS.forEach(l => {
        const opt = document.createElement('option');
        opt.value = l.id;
        opt.textContent = l.label;
        if (l.id === activeLayout) opt.selected = true;
        layoutSelect.appendChild(opt);
    });

    layoutSelect.addEventListener('change', (e) => {
        setLayout(e.target.value);
    });

    layoutGroup.appendChild(layoutLabel);
    layoutGroup.appendChild(layoutSelect);
    body.appendChild(layoutGroup);

    dropdown.appendChild(body);

    container.appendChild(btn);
    container.appendChild(dropdown);

    // Logique d'ouverture / fermeture
    const toggleMenu = (forceState) => {
        const isCurrentlyHidden = dropdown.hasAttribute('hidden');
        const shouldOpen = forceState !== undefined ? forceState : isCurrentlyHidden;

        if (shouldOpen) {
            dropdown.removeAttribute('hidden');
            btn.setAttribute('aria-expanded', 'true');
        } else {
            dropdown.setAttribute('hidden', '');
            btn.setAttribute('aria-expanded', 'false');
        }
    };

    btn.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleMenu();
    });

    // Clic en dehors -> refermer
    document.addEventListener('click', (e) => {
        if (!container.contains(e.target)) {
            toggleMenu(false);
        }
    });

    // Touche Échap -> refermer et redonner le focus au bouton
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !dropdown.hasAttribute('hidden')) {
            toggleMenu(false);
            btn.focus();
        }
    });

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
