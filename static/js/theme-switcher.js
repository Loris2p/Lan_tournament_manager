const THEMES = [
    { id: 'tactical-broadcast', label: 'Tactical Pro' },
    { id: 'neo-arcade', label: 'Neo-Arcade' },
    { id: 'industrial-brutalism', label: 'Industrial' },
    { id: 'stealth-laser', label: 'Stealth Laser' }
];

const LAYOUTS = [
    { id: 'tactical-deck', label: 'Tactical Deck (barre haute)' },
    { id: 'bento-grid', label: 'Bento Grid (cartes flottantes)' }
];

const THEME_KEY = 'ltm_theme';
const LAYOUT_KEY = 'ltm_layout';

export function getActiveTheme() {
    const saved = localStorage.getItem(THEME_KEY);
    if (THEMES.some(t => t.id === saved)) {
        return saved;
    }
    return 'tactical-broadcast';
}

export function getActiveLayout() {
    const saved = localStorage.getItem(LAYOUT_KEY);
    if (LAYOUTS.some(l => l.id === saved)) {
        return saved;
    }
    return 'tactical-deck';
}

export function setTheme(themeId) {
    if (!THEMES.some(t => t.id === themeId)) {
        themeId = 'tactical-broadcast';
    }
    document.documentElement.setAttribute('data-theme', themeId);
    localStorage.setItem(THEME_KEY, themeId);

    const themeStylesheet = document.getElementById('ltm-theme-style');
    if (themeStylesheet) {
        themeStylesheet.href = `/css/themes/${themeId}.css`;
    }

    const selector = document.getElementById('theme-select');
    if (selector && selector.value !== themeId) {
        selector.value = themeId;
    }
}

export function setLayout(layoutId) {
    if (!LAYOUTS.some(l => l.id === layoutId)) {
        layoutId = 'tactical-deck';
    }
    document.documentElement.setAttribute('data-layout', layoutId);
    localStorage.setItem(LAYOUT_KEY, layoutId);

    const selector = document.getElementById('layout-select');
    if (selector && selector.value !== layoutId) {
        selector.value = layoutId;
    }
}

export function initSwitcher() {
    setTheme(getActiveTheme());
    setLayout(getActiveLayout());

    const topbarRight = document.querySelector('.topbar-right');
    if (!topbarRight || document.getElementById('settings-menu-container')) {
        return;
    }

    const container = document.createElement('div');
    container.id = 'settings-menu-container';
    container.className = 'settings-menu-wrapper';

    const btn = document.createElement('button');
    btn.type = 'button';
    btn.id = 'settings-toggle-btn';
    btn.className = 'settings-trigger-btn';
    btn.setAttribute('aria-haspopup', 'true');
    btn.setAttribute('aria-expanded', 'false');
    btn.title = 'Paramètres d\'affichage';
    btn.innerHTML = `<span class="papicon icon-gears" aria-hidden="true"></span><span class="settings-btn-label">Options</span>`;

    const dropdown = document.createElement('div');
    dropdown.id = 'settings-dropdown';
    dropdown.className = 'settings-dropdown';
    dropdown.setAttribute('hidden', '');

    const header = document.createElement('div');
    header.className = 'settings-dropdown-header';
    header.innerHTML = `<span class="settings-dropdown-title"><span class="papicon icon-gears" aria-hidden="true"></span> Paramètres</span>`;
    dropdown.appendChild(header);

    const body = document.createElement('div');
    body.className = 'settings-dropdown-body';

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

    const fsGroup = document.createElement('div');
    fsGroup.className = 'settings-group';

    const fsBtn = document.createElement('button');
    fsBtn.type = 'button';
    fsBtn.id = 'settings-fullscreen-btn';
    fsBtn.className = 'btn btn-outline btn-block btn-sm';
    fsBtn.style.width = '100%';
    fsBtn.style.marginTop = '0.5rem';
    fsBtn.textContent = document.fullscreenElement ? 'Quitter Plein Écran' : 'Mode Plein Écran [F11]';

    fsBtn.addEventListener('click', () => {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch(() => {});
        } else {
            document.exitFullscreen().catch(() => {});
        }
    });

    document.addEventListener('fullscreenchange', () => {
        const isFs = Boolean(document.fullscreenElement);
        fsBtn.textContent = isFs ? 'Quitter Plein Écran' : 'Mode Plein Écran [F11]';
        document.body.classList.toggle('fullscreen-mode', isFs);
    });

    fsGroup.appendChild(fsBtn);
    body.appendChild(fsGroup);

    dropdown.appendChild(body);

    container.appendChild(btn);
    container.appendChild(dropdown);

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

    document.addEventListener('click', (e) => {
        if (!container.contains(e.target)) {
            toggleMenu(false);
        }
    });

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !dropdown.hasAttribute('hidden')) {
            toggleMenu(false);
            btn.focus();
        }
    });

    const wsIndicator = document.getElementById('ws-indicator');
    if (wsIndicator) {
        topbarRight.insertBefore(container, wsIndicator);
    } else {
        topbarRight.prepend(container);
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSwitcher);
} else {
    initSwitcher();
}
