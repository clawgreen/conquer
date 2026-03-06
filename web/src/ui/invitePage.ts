// invitePage.ts — Invite management and landing page (T419-T422)

import { GameClient } from '../network/client';
import { InviteInfo, RACE_NAMES, CLASS_NAMES } from '../types';
import { showAlert } from './modalDialog';

export class InviteManager {
  private container: HTMLDivElement;
  private client: GameClient;
  private gameId: string;
  private onClose: () => void;

  constructor(parent: HTMLElement, client: GameClient, gameId: string, onClose: () => void) {
    this.client = client;
    this.gameId = gameId;
    this.onClose = onClose;

    this.container = document.createElement('div');
    this.container.className = 'lobby';
    this.container.style.cssText = `
      font-family: "Courier New", "Consolas", monospace;
      background: #000; color: #aaa; min-height: 100vh; padding: 20px;
      display: flex; flex-direction: column; align-items: center;
    `;
    parent.appendChild(this.container);
    this.load();
  }

  private async load(): Promise<void> {
    try {
      const invites = await this.client.listInvites(this.gameId);
      this.render(invites);
    } catch (e) {
      this.container.innerHTML = `
        <p style="color:#ff5555;">Error: ${(e as Error).message}</p>
        <button id="inv-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Back</button>`;
      document.getElementById('inv-back')!.addEventListener('click', this.onClose);
    }
  }

  private render(invites: InviteInfo[]): void {
    const inviteRows = invites.map(i => `
      <div style="border:1px solid #333;padding:8px;margin:4px 0;display:flex;justify-content:space-between;align-items:center;">
        <div>
          <span style="color:#55ffff;font-size:16px;">${i.invite_code}</span>
          <span style="color:#555;"> | Uses: ${i.uses}${i.max_uses ? '/' + i.max_uses : '/∞'}</span>
        </div>
        <div>
          <button class="copy-btn" data-code="${i.invite_code}" style="font-family:inherit;background:#222;color:#aaa;border:1px solid #555;padding:4px 8px;cursor:pointer;margin-right:4px;">📋 Copy</button>
          <button class="revoke-btn" data-code="${i.invite_code}" style="font-family:inherit;background:#440000;color:#ff5555;border:1px solid #ff5555;padding:4px 8px;cursor:pointer;">Revoke</button>
        </div>
      </div>
    `).join('');

    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">📨 INVITES</h1>
      <button id="inv-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;position:absolute;top:10px;left:10px;">← Back</button>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Create New Invite</h2>
        <div style="display:flex;gap:8px;align-items:center;margin:8px 0;">
          <label style="color:#aaa;">Max Uses (0=unlimited):</label>
          <input id="inv-max" type="number" min="0" value="0" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
          <label style="color:#aaa;">Expires (hours, 0=never):</label>
          <input id="inv-exp" type="number" min="0" value="0" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>
        <button id="create-invite" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Generate Invite Link</button>
        <div id="invite-msg" style="margin:6px 0;"></div>
      </div>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Active Invites</h2>
        ${inviteRows || '<p style="color:#555;">No invites yet.</p>'}
      </div>
    `;

    document.getElementById('inv-back')!.addEventListener('click', this.onClose);
    document.getElementById('create-invite')!.addEventListener('click', () => this.createInvite());

    this.container.querySelectorAll('.copy-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const code = (e.target as HTMLElement).dataset.code!;
        const url = `${window.location.origin}/invite/${code}`;
        navigator.clipboard.writeText(url).then(() => {
          (e.target as HTMLElement).textContent = '✓ Copied!';
        });
      });
    });

    this.container.querySelectorAll('.revoke-btn').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        const code = (e.target as HTMLElement).dataset.code!;
        // We need invite_id but we only have code — use code-based revoke
        // For now, reload after revoke
        try {
          // Find invite ID — we'll just reload
          await this.client.revokeInvite(this.gameId, code);
          this.load();
        } catch (err) {
          showAlert(`Revoke failed: ${(err as Error).message}`, 'Error');
        }
      });
    });
  }

  private async createInvite(): Promise<void> {
    const maxUses = parseInt((document.getElementById('inv-max') as HTMLInputElement).value) || undefined;
    const expiresHours = parseInt((document.getElementById('inv-exp') as HTMLInputElement).value) || undefined;
    const msg = document.getElementById('invite-msg')!;
    try {
      const invite = await this.client.createInvite(this.gameId, maxUses === 0 ? undefined : maxUses, expiresHours === 0 ? undefined : expiresHours);
      const url = `${window.location.origin}/invite/${invite.invite_code}`;
      msg.style.color = '#55ff55';
      msg.innerHTML = `Created! Link: <a href="${url}" style="color:#55ffff;">${url}</a>`;
      this.load();
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  destroy(): void {
    this.container.remove();
  }
}

// ============================================================
// Invite Landing Page — /invite/:code (T421)
// ============================================================

export class InviteLandingPage {
  private container: HTMLDivElement;
  private client: GameClient;
  private code: string;
  private onJoin: (gameId: string, nationId: number) => void;

  constructor(parent: HTMLElement, client: GameClient, code: string, onJoin: (gameId: string, nationId: number) => void) {
    this.client = client;
    this.code = code;
    this.onJoin = onJoin;

    this.container = document.createElement('div');
    this.container.className = 'lobby';
    this.container.style.cssText = `
      font-family: "Courier New", "Consolas", monospace;
      background: #000; color: #aaa; min-height: 100vh; padding: 20px;
      display: flex; flex-direction: column; align-items: center;
    `;
    parent.appendChild(this.container);
    this.load();
  }

  private async load(): Promise<void> {
    try {
      const invite = await this.client.getInvite(this.code);
      this.render(invite);
    } catch (e) {
      this.container.innerHTML = `
        <h1 style="color:#ff5555;">Invalid Invite</h1>
        <p style="color:#aaa;">${(e as Error).message}</p>
        <p style="color:#555;">This invite may have expired or been revoked.</p>
      `;
    }
  }

  private render(invite: InviteInfo): void {
    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">⚔ CONQUER</h1>
      <h2 style="color:#ffff55;">You've been invited to:</h2>
      <div style="border:1px solid #55ff55;padding:20px;margin:20px;max-width:400px;width:100%;">
        <div style="font-size:24px;color:#55ffff;text-align:center;">${invite.game_name}</div>
        <div style="text-align:center;color:#555;margin:4px 0;">Code: ${invite.invite_code}</div>
      </div>

      <div style="max-width:400px;width:100%;margin:10px 0;">
        <h2 style="color:#ffff55;">Create Your Nation</h2>
        <div style="margin:6px 0;">
          <label style="color:#aaa;">Nation Name:</label>
          <input id="inv-nation" type="text" maxlength="9" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:6px 10px;width:100%;box-sizing:border-box;">
        </div>
        <div style="margin:6px 0;">
          <label style="color:#aaa;">Leader Name:</label>
          <input id="inv-leader" type="text" maxlength="9" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:6px 10px;width:100%;box-sizing:border-box;">
        </div>
        <div style="margin:6px 0;">
          <label style="color:#aaa;">Race:</label>
          <select id="inv-race" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:6px 10px;width:100%;">
            <option value="H">Human</option><option value="E">Elf</option><option value="D">Dwarf</option><option value="O">Orc</option>
          </select>
        </div>
        <div style="margin:6px 0;">
          <label style="color:#aaa;">Class:</label>
          <select id="inv-class" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:6px 10px;width:100%;">
            <option value="1">King</option><option value="2">Emperor</option><option value="3">Wizard</option>
            <option value="4">Priest</option><option value="6">Trader</option><option value="7">Warlord</option>
          </select>
        </div>
        <button id="inv-join" style="font-family:inherit;background:#004400;color:#55ff55;border:1px solid #55ff55;padding:10px 24px;cursor:pointer;font-size:16px;width:100%;margin-top:10px;">⚔ Join Game</button>
        <div id="inv-error" style="color:#ff5555;margin:6px 0;"></div>
      </div>
    `;

    document.getElementById('inv-join')!.addEventListener('click', () => this.joinViaInvite());
  }

  private async joinViaInvite(): Promise<void> {
    const nation = (document.getElementById('inv-nation') as HTMLInputElement).value;
    const leader = (document.getElementById('inv-leader') as HTMLInputElement).value;
    const race = (document.getElementById('inv-race') as HTMLSelectElement).value;
    const classId = parseInt((document.getElementById('inv-class') as HTMLSelectElement).value);
    const errorEl = document.getElementById('inv-error')!;

    if (!nation || !leader) { errorEl.textContent = 'Fill in nation and leader names'; return; }

    try {
      const resp = await this.client.acceptInvite(this.code, nation, leader, race, classId, '*');
      this.onJoin(resp.game_id, resp.nation_id);
    } catch (e) {
      errorEl.textContent = `Failed: ${(e as Error).message}`;
    }
  }

  destroy(): void {
    this.container.remove();
  }
}
