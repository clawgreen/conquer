// modalDialog.ts — Themed modal dialogs replacing window.alert/confirm

const MODAL_CSS = `
  position: fixed; inset: 0; z-index: 2000;
  background: rgba(0,0,0,0.8); display: flex;
  align-items: center; justify-content: center;
  animation: modalFadeIn 0.15s ease-out;
`;

const BOX_CSS = `
  background: #111; border: 2px solid #333; border-radius: 8px;
  padding: 24px; max-width: 420px; width: 90vw;
  font-family: "Courier New", monospace; color: #ccc;
  box-shadow: 0 0 30px rgba(0,100,0,0.3);
`;

function injectAnimation(): void {
  if (document.getElementById('modal-anim-style')) return;
  const s = document.createElement('style');
  s.id = 'modal-anim-style';
  s.textContent = `@keyframes modalFadeIn { from { opacity: 0; } to { opacity: 1; } }`;
  document.head.appendChild(s);
}

function createOverlay(): HTMLDivElement {
  injectAnimation();
  const overlay = document.createElement('div');
  overlay.style.cssText = MODAL_CSS;
  return overlay;
}

function btnStyle(color: string, bg: string = '#222'): string {
  return `font-family:inherit;background:${bg};color:${color};border:1px solid ${color};padding:10px 20px;cursor:pointer;font-size:15px;border-radius:4px;min-width:80px;`;
}

/** Show a themed alert dialog (replaces window.alert) */
export function showAlert(message: string, title?: string): Promise<void> {
  return new Promise(resolve => {
    const overlay = createOverlay();
    const box = document.createElement('div');
    box.style.cssText = BOX_CSS;

    if (title) {
      const h = document.createElement('h3');
      h.style.cssText = 'margin:0 0 12px;color:#55ff55;font-size:16px;';
      h.textContent = title;
      box.appendChild(h);
    }

    const msg = document.createElement('p');
    msg.style.cssText = 'margin:0 0 20px;font-size:15px;line-height:1.5;';
    msg.textContent = message;
    box.appendChild(msg);

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display:flex;justify-content:flex-end;';
    const okBtn = document.createElement('button');
    okBtn.textContent = 'OK';
    okBtn.style.cssText = btnStyle('#55ff55');
    okBtn.onclick = () => { overlay.remove(); resolve(); };
    btnRow.appendChild(okBtn);
    box.appendChild(btnRow);

    overlay.appendChild(box);
    document.body.appendChild(overlay);
    okBtn.focus();

    // Escape to dismiss
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Enter') {
        e.preventDefault();
        document.removeEventListener('keydown', onKey, true);
        overlay.remove();
        resolve();
      }
    };
    document.addEventListener('keydown', onKey, true);
  });
}

/** Show a themed confirm dialog (replaces window.confirm) */
export function showConfirm(
  message: string,
  options?: { title?: string; confirmText?: string; cancelText?: string; danger?: boolean }
): Promise<boolean> {
  const { title, confirmText = 'Confirm', cancelText = 'Cancel', danger = false } = options ?? {};

  return new Promise(resolve => {
    const overlay = createOverlay();
    const box = document.createElement('div');
    box.style.cssText = BOX_CSS;

    if (title) {
      const h = document.createElement('h3');
      h.style.cssText = `margin:0 0 12px;color:${danger ? '#ff5555' : '#55ff55'};font-size:16px;`;
      h.textContent = title;
      box.appendChild(h);
    }

    const msg = document.createElement('p');
    msg.style.cssText = 'margin:0 0 20px;font-size:15px;line-height:1.5;';
    msg.textContent = message;
    box.appendChild(msg);

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display:flex;justify-content:flex-end;gap:12px;';

    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = cancelText;
    cancelBtn.style.cssText = btnStyle('#888');
    cancelBtn.onclick = () => { cleanup(); resolve(false); };

    const confirmBtn = document.createElement('button');
    confirmBtn.textContent = confirmText;
    confirmBtn.style.cssText = danger
      ? btnStyle('#ff3333', '#330000')
      : btnStyle('#55ff55', '#003300');
    confirmBtn.onclick = () => { cleanup(); resolve(true); };

    btnRow.appendChild(cancelBtn);
    btnRow.appendChild(confirmBtn);
    box.appendChild(btnRow);

    overlay.appendChild(box);
    document.body.appendChild(overlay);
    confirmBtn.focus();

    const cleanup = () => { 
      document.removeEventListener('keydown', onKey, true);
      overlay.remove(); 
    };

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        cleanup();
        resolve(false);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        cleanup();
        resolve(true);
      }
    };
    document.addEventListener('keydown', onKey, true);

    // Click overlay to cancel
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) { cleanup(); resolve(false); }
    });
  });
}
