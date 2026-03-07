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

/** Show an input dialog — returns entered string or null on cancel */
export function showInput(
  message: string,
  options?: { title?: string; defaultValue?: string; placeholder?: string; inputType?: string; confirmText?: string }
): Promise<string | null> {
  const { title, defaultValue = '', placeholder = '', inputType = 'text', confirmText = 'OK' } = options ?? {};
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
    msg.style.cssText = 'margin:0 0 12px;font-size:15px;line-height:1.5;';
    msg.textContent = message;
    box.appendChild(msg);

    const input = document.createElement('input');
    input.type = inputType;
    input.value = defaultValue;
    input.placeholder = placeholder;
    input.style.cssText = 'width:100%;box-sizing:border-box;background:#222;color:#ccc;border:1px solid #555;padding:8px;font-family:inherit;font-size:14px;border-radius:4px;margin-bottom:16px;';
    box.appendChild(input);

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display:flex;justify-content:flex-end;gap:12px;';
    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = 'Cancel';
    cancelBtn.style.cssText = btnStyle('#888');
    cancelBtn.onclick = () => { cleanup(); resolve(null); };
    const okBtn = document.createElement('button');
    okBtn.textContent = confirmText;
    okBtn.style.cssText = btnStyle('#55ff55', '#003300');
    okBtn.onclick = () => { cleanup(); resolve(input.value); };
    btnRow.appendChild(cancelBtn);
    btnRow.appendChild(okBtn);
    box.appendChild(btnRow);

    overlay.appendChild(box);
    document.body.appendChild(overlay);
    input.focus();
    input.select();

    const cleanup = () => { document.removeEventListener('keydown', onKey, true); overlay.remove(); };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { e.preventDefault(); cleanup(); resolve(null); }
      else if (e.key === 'Enter') { e.preventDefault(); cleanup(); resolve(input.value); }
    };
    document.addEventListener('keydown', onKey, true);
    overlay.addEventListener('click', (e) => { if (e.target === overlay) { cleanup(); resolve(null); } });
  });
}

/** Show a select dialog — returns selected index or -1 on cancel */
export function showSelect(
  message: string,
  items: { label: string; detail?: string; disabled?: boolean }[],
  options?: { title?: string; confirmText?: string }
): Promise<number> {
  const { title, confirmText = 'Select' } = options ?? {};
  return new Promise(resolve => {
    const overlay = createOverlay();
    const box = document.createElement('div');
    box.style.cssText = BOX_CSS + 'max-height:80vh;overflow-y:auto;';

    if (title) {
      const h = document.createElement('h3');
      h.style.cssText = 'margin:0 0 12px;color:#55ff55;font-size:16px;';
      h.textContent = title;
      box.appendChild(h);
    }

    const msg = document.createElement('p');
    msg.style.cssText = 'margin:0 0 12px;font-size:15px;line-height:1.5;';
    msg.textContent = message;
    box.appendChild(msg);

    let selected = items.findIndex(i => !i.disabled);

    const listDiv = document.createElement('div');
    listDiv.style.cssText = 'max-height:300px;overflow-y:auto;margin-bottom:16px;';

    const renderList = () => {
      listDiv.innerHTML = '';
      items.forEach((item, idx) => {
        const row = document.createElement('div');
        row.style.cssText = `padding:8px 12px;cursor:${item.disabled ? 'default' : 'pointer'};border-radius:4px;margin-bottom:2px;font-size:14px;` +
          (item.disabled ? 'opacity:0.4;' : '') +
          (idx === selected ? 'background:#003300;border:1px solid #55ff55;' : 'background:#1a1a1a;border:1px solid transparent;');
        row.textContent = item.label;
        if (item.detail) {
          const det = document.createElement('div');
          det.style.cssText = 'font-size:12px;opacity:0.6;margin-top:2px;';
          det.textContent = item.detail;
          row.appendChild(det);
        }
        if (!item.disabled) {
          row.onclick = () => { selected = idx; renderList(); };
          row.ondblclick = () => { cleanup(); resolve(idx); };
        }
        listDiv.appendChild(row);
      });
    };
    renderList();
    box.appendChild(listDiv);

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display:flex;justify-content:flex-end;gap:12px;';
    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = 'Cancel';
    cancelBtn.style.cssText = btnStyle('#888');
    cancelBtn.onclick = () => { cleanup(); resolve(-1); };
    const okBtn = document.createElement('button');
    okBtn.textContent = confirmText;
    okBtn.style.cssText = btnStyle('#55ff55', '#003300');
    okBtn.onclick = () => { cleanup(); resolve(selected); };
    btnRow.appendChild(cancelBtn);
    btnRow.appendChild(okBtn);
    box.appendChild(btnRow);

    overlay.appendChild(box);
    document.body.appendChild(overlay);
    okBtn.focus();

    const cleanup = () => { document.removeEventListener('keydown', onKey, true); overlay.remove(); };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { e.preventDefault(); cleanup(); resolve(-1); }
      else if (e.key === 'Enter') { e.preventDefault(); cleanup(); resolve(selected); }
      else if (e.key === 'ArrowDown') { e.preventDefault(); for (let i = selected + 1; i < items.length; i++) { if (!items[i].disabled) { selected = i; renderList(); break; } } }
      else if (e.key === 'ArrowUp') { e.preventDefault(); for (let i = selected - 1; i >= 0; i--) { if (!items[i].disabled) { selected = i; renderList(); break; } } }
    };
    document.addEventListener('keydown', onKey, true);
    overlay.addEventListener('click', (e) => { if (e.target === overlay) { cleanup(); resolve(-1); } });
  });
}

/** Show a form dialog with multiple fields — returns record or null on cancel */
export function showForm(
  fields: { id: string; label: string; type: 'text' | 'number' | 'select' | 'range'; defaultValue?: string; options?: { label: string; value: string }[]; min?: number; max?: number; step?: number }[],
  options?: { title?: string; confirmText?: string; message?: string }
): Promise<Record<string, string> | null> {
  const { title, confirmText = 'Submit', message } = options ?? {};
  return new Promise(resolve => {
    const overlay = createOverlay();
    const box = document.createElement('div');
    box.style.cssText = BOX_CSS + 'max-height:80vh;overflow-y:auto;';

    if (title) {
      const h = document.createElement('h3');
      h.style.cssText = 'margin:0 0 12px;color:#55ff55;font-size:16px;';
      h.textContent = title;
      box.appendChild(h);
    }
    if (message) {
      const msg = document.createElement('p');
      msg.style.cssText = 'margin:0 0 12px;font-size:14px;opacity:0.8;';
      msg.textContent = message;
      box.appendChild(msg);
    }

    const inputs: Record<string, HTMLInputElement | HTMLSelectElement> = {};
    for (const f of fields) {
      const row = document.createElement('div');
      row.style.cssText = 'margin-bottom:12px;';
      const lbl = document.createElement('label');
      lbl.style.cssText = 'display:block;font-size:13px;color:#888;margin-bottom:4px;';
      lbl.textContent = f.label;
      row.appendChild(lbl);

      if (f.type === 'select' && f.options) {
        const sel = document.createElement('select');
        sel.style.cssText = 'width:100%;box-sizing:border-box;background:#222;color:#ccc;border:1px solid #555;padding:8px;font-family:inherit;font-size:14px;border-radius:4px;';
        for (const opt of f.options) {
          const o = document.createElement('option');
          o.value = opt.value;
          o.textContent = opt.label;
          if (opt.value === f.defaultValue) o.selected = true;
          sel.appendChild(o);
        }
        row.appendChild(sel);
        inputs[f.id] = sel;
      } else {
        const inp = document.createElement('input');
        inp.type = f.type === 'range' ? 'range' : f.type;
        inp.value = f.defaultValue ?? '';
        if (f.min !== undefined) inp.min = String(f.min);
        if (f.max !== undefined) inp.max = String(f.max);
        if (f.step !== undefined) inp.step = String(f.step);
        inp.style.cssText = 'width:100%;box-sizing:border-box;background:#222;color:#ccc;border:1px solid #555;padding:8px;font-family:inherit;font-size:14px;border-radius:4px;';
        row.appendChild(inp);
        inputs[f.id] = inp;
      }
      box.appendChild(row);
    }

    const btnRow = document.createElement('div');
    btnRow.style.cssText = 'display:flex;justify-content:flex-end;gap:12px;margin-top:16px;';
    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = 'Cancel';
    cancelBtn.style.cssText = btnStyle('#888');
    cancelBtn.onclick = () => { cleanup(); resolve(null); };
    const okBtn = document.createElement('button');
    okBtn.textContent = confirmText;
    okBtn.style.cssText = btnStyle('#55ff55', '#003300');
    okBtn.onclick = () => {
      const result: Record<string, string> = {};
      for (const f of fields) result[f.id] = inputs[f.id].value;
      cleanup(); resolve(result);
    };
    btnRow.appendChild(cancelBtn);
    btnRow.appendChild(okBtn);
    box.appendChild(btnRow);

    overlay.appendChild(box);
    document.body.appendChild(overlay);

    // Focus first input
    const firstInput = Object.values(inputs)[0];
    if (firstInput) firstInput.focus();

    const cleanup = () => { document.removeEventListener('keydown', onKey, true); overlay.remove(); };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { e.preventDefault(); cleanup(); resolve(null); }
    };
    document.addEventListener('keydown', onKey, true);
    overlay.addEventListener('click', (e) => { if (e.target === overlay) { cleanup(); resolve(null); } });
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
