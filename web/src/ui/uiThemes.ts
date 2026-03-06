// uiThemes.ts — UI chrome themes (separate from game map themes)

export interface UiTheme {
  id: string;
  name: string;
  // Bezel / monitor frame
  bezelBg: string;
  bezelBorder: string;
  bezelShadow: string;
  bezelRadius: string;
  screenGlow: string;      // box-shadow on the canvas area
  // Sidebar chrome
  sidebarBg: string;
  sidebarBorder: string;
  sidebarText: string;
  sidebarDim: string;
  sidebarAccent: string;
  sidebarHeaderBg: string;
  // Buttons
  btnBg: string;
  btnText: string;
  btnBorder: string;
  btnActiveBg: string;
  btnActiveText: string;
  btnHoverBg: string;
  // Header bar
  headerBg: string;
  headerText: string;
  headerBorder: string;
}

export const UI_THEMES: Record<string, UiTheme> = {
  terminal: {
    id: 'terminal',
    name: 'Terminal',
    bezelBg: '#0a0a0a',
    bezelBorder: '#222',
    bezelShadow: '0 0 30px rgba(0,50,0,0.3)',
    bezelRadius: '8px',
    screenGlow: '0 0 15px rgba(0,255,0,0.08) inset',
    sidebarBg: '#0a0a0a',
    sidebarBorder: '#1a1a1a',
    sidebarText: '#55ff55',
    sidebarDim: '#336633',
    sidebarAccent: '#55ff55',
    sidebarHeaderBg: '#0f1a0f',
    btnBg: '#0a1a0a',
    btnText: '#55ff55',
    btnBorder: '#224422',
    btnActiveBg: '#003300',
    btnActiveText: '#aaffaa',
    btnHoverBg: '#112211',
    headerBg: '#0a0a0a',
    headerText: '#55ff55',
    headerBorder: '#1a1a1a',
  },
  slate: {
    id: 'slate',
    name: 'Slate',
    bezelBg: '#1a1d21',
    bezelBorder: '#2a2d31',
    bezelShadow: '0 4px 20px rgba(0,0,0,0.5)',
    bezelRadius: '6px',
    screenGlow: 'none',
    sidebarBg: '#1a1d21',
    sidebarBorder: '#2a2d31',
    sidebarText: '#c8ccd0',
    sidebarDim: '#5a5e62',
    sidebarAccent: '#6ab0f3',
    sidebarHeaderBg: '#22262a',
    btnBg: '#22262a',
    btnText: '#c8ccd0',
    btnBorder: '#333740',
    btnActiveBg: '#2a4060',
    btnActiveText: '#6ab0f3',
    btnHoverBg: '#2a2e32',
    headerBg: '#1a1d21',
    headerText: '#c8ccd0',
    headerBorder: '#2a2d31',
  },
  amber: {
    id: 'amber',
    name: 'Amber CRT',
    bezelBg: '#0a0804',
    bezelBorder: '#1a1408',
    bezelShadow: '0 0 30px rgba(50,30,0,0.3)',
    bezelRadius: '12px',
    screenGlow: '0 0 15px rgba(255,180,0,0.06) inset',
    sidebarBg: '#0a0804',
    sidebarBorder: '#1a1408',
    sidebarText: '#cc8800',
    sidebarDim: '#553300',
    sidebarAccent: '#ffbb33',
    sidebarHeaderBg: '#0f0a04',
    btnBg: '#0f0a04',
    btnText: '#cc8800',
    btnBorder: '#332200',
    btnActiveBg: '#221500',
    btnActiveText: '#ffbb33',
    btnHoverBg: '#1a1008',
    headerBg: '#0a0804',
    headerText: '#cc8800',
    headerBorder: '#1a1408',
  },
  military: {
    id: 'military',
    name: 'Military',
    bezelBg: '#0c0f0c',
    bezelBorder: '#1a221a',
    bezelShadow: '0 2px 15px rgba(0,0,0,0.4)',
    bezelRadius: '4px',
    screenGlow: 'none',
    sidebarBg: '#0c0f0c',
    sidebarBorder: '#1a221a',
    sidebarText: '#88aa88',
    sidebarDim: '#445544',
    sidebarAccent: '#66cc66',
    sidebarHeaderBg: '#101510',
    btnBg: '#101510',
    btnText: '#88aa88',
    btnBorder: '#223322',
    btnActiveBg: '#1a2a1a',
    btnActiveText: '#66cc66',
    btnHoverBg: '#151a15',
    headerBg: '#0c0f0c',
    headerText: '#88aa88',
    headerBorder: '#1a221a',
  },
};

export function getUiTheme(id: string): UiTheme {
  return UI_THEMES[id] ?? UI_THEMES.terminal;
}

/** Apply UI theme as CSS custom properties on :root for menu styling */
export function applyUiThemeCss(id: string): void {
  const t = getUiTheme(id);
  const root = document.documentElement.style;
  root.setProperty('--ui-bezel-bg', t.bezelBg);
  root.setProperty('--ui-bezel-border', t.bezelBorder);
  root.setProperty('--ui-bezel-shadow', t.bezelShadow);
  root.setProperty('--ui-bezel-radius', t.bezelRadius);
  root.setProperty('--ui-screen-glow', t.screenGlow);
  root.setProperty('--ui-sidebar-bg', t.sidebarBg);
  root.setProperty('--ui-sidebar-border', t.sidebarBorder);
  root.setProperty('--ui-sidebar-text', t.sidebarText);
  root.setProperty('--ui-sidebar-dim', t.sidebarDim);
  root.setProperty('--ui-sidebar-accent', t.sidebarAccent);
  root.setProperty('--ui-sidebar-header-bg', t.sidebarHeaderBg);
  root.setProperty('--ui-btn-bg', t.btnBg);
  root.setProperty('--ui-btn-text', t.btnText);
  root.setProperty('--ui-btn-border', t.btnBorder);
  root.setProperty('--ui-btn-active-bg', t.btnActiveBg);
  root.setProperty('--ui-btn-active-text', t.btnActiveText);
  root.setProperty('--ui-btn-hover-bg', t.btnHoverBg);
  root.setProperty('--ui-header-bg', t.headerBg);
  root.setProperty('--ui-header-text', t.headerText);
  root.setProperty('--ui-header-border', t.headerBorder);
}

export const ALL_UI_THEMES = Object.values(UI_THEMES);
