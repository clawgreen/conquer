// profilePage.ts — User profile page (T409-T411, T434)

import { GameClient } from '../network/client';
import { UserProfile, GameHistoryEntry, NotificationPreferences, CLASS_NAMES, RACE_NAMES } from '../types';

export class ProfilePage {
  private container: HTMLDivElement;
  private client: GameClient;
  private onClose: () => void;

  constructor(parent: HTMLElement, client: GameClient, onClose: () => void) {
    this.client = client;
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
      const [profile, prefs] = await Promise.all([
        this.client.getProfile(),
        this.client.getNotificationPreferences(),
      ]);
      this.render(profile, prefs);
    } catch (e) {
      this.container.innerHTML = `<p style="color:#ff5555;">Failed to load profile: ${(e as Error).message}</p>
        <button id="profile-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Back</button>`;
      document.getElementById('profile-back')!.addEventListener('click', this.onClose);
    }
  }

  private render(profile: UserProfile, prefs: NotificationPreferences): void {
    const historyRows = profile.game_history.map(h => {
      const rn = RACE_NAMES[h.race] || h.race;
      const cn = CLASS_NAMES[h.class] || `class ${h.class}`;
      const outcome = h.outcome === 'won' ? '<span style="color:#55ff55;">WON</span>'
        : h.outcome === 'lost' ? '<span style="color:#ff5555;">LOST</span>'
        : h.outcome === 'eliminated' ? '<span style="color:#ff5555;">ELIMINATED</span>'
        : '<span style="color:#ffff55;">ACTIVE</span>';
      return `<tr>
        <td style="padding:2px 8px;color:#55ffff;">${h.game_name}</td>
        <td style="padding:2px 8px;">${h.nation_name}</td>
        <td style="padding:2px 8px;">${rn} ${cn}</td>
        <td style="padding:2px 8px;text-align:right;">${h.final_score}</td>
        <td style="padding:2px 8px;">${outcome}</td>
      </tr>`;
    }).join('');

    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">⚡ PROFILE</h1>
      <button id="profile-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;position:absolute;top:10px;left:10px;">← Back</button>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Account Info</h2>
        <table style="width:100%;">
          <tr><td style="color:#aaa;padding:4px 8px;">Username:</td><td style="color:#55ffff;">${profile.username}</td></tr>
          <tr><td style="color:#aaa;padding:4px 8px;">Display Name:</td><td><input id="prof-name" type="text" value="${profile.display_name}" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:200px;"></td></tr>
          <tr><td style="color:#aaa;padding:4px 8px;">Email:</td><td><input id="prof-email" type="text" value="${profile.email}" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:200px;"></td></tr>
          <tr><td style="color:#aaa;padding:4px 8px;">Member Since:</td><td>${new Date(profile.created_at).toLocaleDateString()}</td></tr>
        </table>
        <button id="save-profile" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;margin:8px 4px;">Save Profile</button>
        <div id="profile-msg" style="color:#55ff55;margin:4px 0;"></div>
      </div>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Change Password</h2>
        <div style="margin:4px 0;"><input id="old-pass" type="password" placeholder="Current password" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:200px;"></div>
        <div style="margin:4px 0;"><input id="new-pass" type="password" placeholder="New password" style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:200px;"></div>
        <button id="change-pass" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;margin:8px 4px;">Change Password</button>
        <div id="pass-msg" style="margin:4px 0;"></div>
      </div>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Stats</h2>
        <div style="display:flex;gap:20px;">
          <div style="text-align:center;"><div style="font-size:24px;color:#55ff55;">${profile.games_played}</div><div>Games</div></div>
          <div style="text-align:center;"><div style="font-size:24px;color:#55ff55;">${profile.games_won}</div><div>Wins</div></div>
          <div style="text-align:center;"><div style="font-size:24px;color:#ff5555;">${profile.games_lost}</div><div>Losses</div></div>
        </div>
      </div>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Game History</h2>
        ${profile.game_history.length === 0
          ? '<p style="color:#555;">No games played yet.</p>'
          : `<table style="width:100%;border-collapse:collapse;">
              <tr style="border-bottom:1px solid #333;">
                <th style="text-align:left;padding:4px 8px;color:#aaa;">Game</th>
                <th style="text-align:left;padding:4px 8px;color:#aaa;">Nation</th>
                <th style="text-align:left;padding:4px 8px;color:#aaa;">Race/Class</th>
                <th style="text-align:right;padding:4px 8px;color:#aaa;">Score</th>
                <th style="text-align:left;padding:4px 8px;color:#aaa;">Outcome</th>
              </tr>
              ${historyRows}
            </table>`
        }
      </div>

      <div style="max-width:600px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Notification Preferences</h2>
        <div style="display:grid;gap:6px;">
          ${this.prefToggle('pref-your-turn', 'Your Turn', prefs.your_turn)}
          ${this.prefToggle('pref-game-started', 'Game Started', prefs.game_started)}
          ${this.prefToggle('pref-game-invite', 'Game Invite', prefs.game_invite)}
          ${this.prefToggle('pref-under-attack', 'Under Attack', prefs.under_attack)}
          ${this.prefToggle('pref-turn-advanced', 'Turn Advanced', prefs.turn_advanced)}
          ${this.prefToggle('pref-player-joined', 'Player Joined', prefs.player_joined)}
          ${this.prefToggle('pref-game-completed', 'Game Completed', prefs.game_completed)}
        </div>
        <button id="save-prefs" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;margin:8px 4px;">Save Preferences</button>
        <div id="prefs-msg" style="color:#55ff55;margin:4px 0;"></div>
      </div>
    `;

    document.getElementById('profile-back')!.addEventListener('click', this.onClose);
    document.getElementById('save-profile')!.addEventListener('click', () => this.saveProfile());
    document.getElementById('change-pass')!.addEventListener('click', () => this.changePassword());
    document.getElementById('save-prefs')!.addEventListener('click', () => this.savePrefs());
  }

  private prefToggle(id: string, label: string, checked: boolean): string {
    return `<label style="display:flex;align-items:center;gap:8px;cursor:pointer;">
      <input type="checkbox" id="${id}" ${checked ? 'checked' : ''} style="accent-color:#55ff55;">
      <span>${label}</span>
    </label>`;
  }

  private async saveProfile(): Promise<void> {
    const name = (document.getElementById('prof-name') as HTMLInputElement).value;
    const email = (document.getElementById('prof-email') as HTMLInputElement).value;
    const msg = document.getElementById('profile-msg')!;
    try {
      await this.client.updateProfile({ display_name: name, email });
      msg.style.color = '#55ff55';
      msg.textContent = 'Profile updated!';
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  private async changePassword(): Promise<void> {
    const oldPass = (document.getElementById('old-pass') as HTMLInputElement).value;
    const newPass = (document.getElementById('new-pass') as HTMLInputElement).value;
    const msg = document.getElementById('pass-msg')!;
    try {
      await this.client.changePassword(oldPass, newPass);
      msg.style.color = '#55ff55';
      msg.textContent = 'Password changed!';
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  private async savePrefs(): Promise<void> {
    const prefs: NotificationPreferences = {
      your_turn: (document.getElementById('pref-your-turn') as HTMLInputElement).checked,
      game_started: (document.getElementById('pref-game-started') as HTMLInputElement).checked,
      game_invite: (document.getElementById('pref-game-invite') as HTMLInputElement).checked,
      under_attack: (document.getElementById('pref-under-attack') as HTMLInputElement).checked,
      turn_advanced: (document.getElementById('pref-turn-advanced') as HTMLInputElement).checked,
      player_joined: (document.getElementById('pref-player-joined') as HTMLInputElement).checked,
      game_completed: (document.getElementById('pref-game-completed') as HTMLInputElement).checked,
      email_enabled: false,
    };
    const msg = document.getElementById('prefs-msg')!;
    try {
      await this.client.setNotificationPreferences(prefs);
      msg.style.color = '#55ff55';
      msg.textContent = 'Preferences saved!';
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  destroy(): void {
    this.container.remove();
  }
}
