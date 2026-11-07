const STYLES = [
    { id: 'arena', label: 'Arena (affiche & dock)', layout: 'arena' },
    { id: 'tactical-pro', label: 'Tactical Pro (barre haute)', layout: 'tactical-deck' }
];

const THEMES = [
    { id: 'tactical-broadcast', label: 'Tactical Pro' }
];

const LAYOUTS = [
    { id: 'arena', label: 'Arena' },
    { id: 'tactical-deck', label: 'Tactical Pro' }
];

const THEME_KEY = 'ltm_theme';
const LAYOUT_KEY = 'ltm_layout';

export function getActiveTheme() {
    return 'tactical-broadcast';
}

export function getActiveLayout() {
    const saved = localStorage.getItem(LAYOUT_KEY);
    if (saved === 'tactical-deck' || saved === 'arena') {
        return saved;
    }
    return 'arena';
}

export function setTheme(themeId) {
    themeId = 'tactical-broadcast';
    document.documentElement.setAttribute('data-theme', themeId);
    localStorage.setItem(THEME_KEY, themeId);

    const themeStylesheet = document.getElementById('ltm-theme-style');
    if (themeStylesheet) {
        themeStylesheet.href = `/css/themes/${themeId}.css`;
    }
}

export function setLayout(layoutId) {
    if (layoutId !== 'tactical-deck' && layoutId !== 'arena') {
        layoutId = 'arena';
    }
    document.documentElement.setAttribute('data-layout', layoutId);
    localStorage.setItem(LAYOUT_KEY, layoutId);
    // Le serveur choisit le markup des pages publiques d'après ce cookie.
    document.cookie = `${LAYOUT_KEY}=${layoutId}; path=/; max-age=31536000; SameSite=Lax`;
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
    btn.innerHTML = `<span class="papicon icon-gears" aria-hidden="true"></span>`;

    const dropdown = document.createElement('div');
    dropdown.id = 'settings-dropdown';
    dropdown.className = 'settings-dropdown';
    dropdown.setAttribute('hidden', '');

    const header = document.createElement('div');
    header.className = 'settings-dropdown-header';
    header.innerHTML = `<span class="settings-dropdown-title"><span class="papicon icon-gears" aria-hidden="true"></span> Paramètres</span>`;

    const wsIndicator = document.getElementById('ws-indicator');
    if (wsIndicator) {
        header.appendChild(wsIndicator);
    }
    dropdown.appendChild(header);

    const body = document.createElement('div');
    body.className = 'settings-dropdown-body';

    const styleGroup = document.createElement('div');
    styleGroup.className = 'settings-group';

    const styleLabel = document.createElement('label');
    styleLabel.htmlFor = 'theme-select';
    styleLabel.className = 'settings-label';
    styleLabel.innerHTML = `<span class="papicon icon-palette" aria-hidden="true"></span><span>Thème</span>`;

    const styleSelect = document.createElement('select');
    styleSelect.id = 'theme-select';
    styleSelect.className = 'switcher-select-input';
    styleSelect.title = 'Changer le thème du tournoi';

    const currentLayout = getActiveLayout();
    const activeStyleId = currentLayout === 'tactical-deck' ? 'tactical-pro' : 'arena';

    STYLES.forEach(s => {
        const opt = document.createElement('option');
        opt.value = s.id;
        opt.textContent = s.label;
        if (s.id === activeStyleId) opt.selected = true;
        styleSelect.appendChild(opt);
    });

    styleSelect.addEventListener('change', (e) => {
        const wasArena = document.documentElement.getAttribute('data-layout') === 'arena';
        const targetLayout = e.target.value === 'tactical-pro' ? 'tactical-deck' : 'arena';
        setLayout(targetLayout);
        setTheme('tactical-broadcast');
        if (wasArena !== (targetLayout === 'arena')) {
            window.location.reload();
        }
    });

    styleGroup.appendChild(styleLabel);
    styleGroup.appendChild(styleSelect);
    body.appendChild(styleGroup);

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

    const authBtn = document.getElementById('auth-btn') || document.getElementById('logout-btn');
    if (authBtn) {
        topbarRight.insertBefore(container, authBtn);
    } else {
        topbarRight.appendChild(container);
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSwitcher);
} else {
    initSwitcher();
}
