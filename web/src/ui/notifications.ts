// notifications.ts — In-app notification dropdown (T432-T434)

import { GameClient } from '../network/client';
import { Notification } from '../types';

export class NotificationBell {
  private container: HTMLDivElement;
  private bell: HTMLButtonElement;
  private dropdown: HTMLDivElement;
  private client: GameClient;
  private unreadCount = 0;
  private visible = false;
  private pollTimer: number = 0;

  constructor(parent: HTMLElement, client: GameClient) {
    this.client = client;

    this.container = document.createElement('div');
    this.container.style.cssText = `position:relative;z-index:200;display:inline-block;`;

    this.bell = document.createElement('button');
    this.bell.style.cssText = `
      font-family: "Courier New", monospace; font-size: 18px;
      background: #111; color: #ffff55; border: 1px solid #333;
      padding: 4px 10px; cursor: pointer; position: relative;
    `;
    this.bell.textContent = '🔔';
    this.bell.addEventListener('click', () => this.toggle());
    this.container.appendChild(this.bell);

    this.dropdown = document.createElement('div');
    this.dropdown.style.cssText = `
      display: none; position: absolute; top: 100%; right: 0;
      width: 320px; max-height: 400px; overflow-y: auto;
      background: #0a0a0a; border: 1px solid #333;
      font-family: "Courier New", monospace; font-size: 12px;
    `;
    this.container.appendChild(this.dropdown);

    parent.appendChild(this.container);

    // Poll for notifications every 30s
    this.refresh();
    this.pollTimer = window.setInterval(() => this.refresh(), 30000);
  }

  private async refresh(): Promise<void> {
    try {
      const notifs = await this.client.getNotifications(true);
      this.unreadCount = notifs.length;
      this.updateBell();
      if (this.visible) {
        this.renderDropdown(notifs);
      }
    } catch {
      // Ignore errors during polling
    }
  }

  private updateBell(): void {
    this.bell.innerHTML = this.unreadCount > 0
      ? `🔔<span style="position:absolute;top:-4px;right:-4px;background:#ff5555;color:#fff;border-radius:50%;font-size:10px;padding:1px 4px;min-width:12px;text-align:center;">${this.unreadCount}</span>`
      : '🔔';
  }

  private async toggle(): Promise<void> {
    this.visible = !this.visible;
    if (this.visible) {
      this.dropdown.style.display = 'block';
      try {
        const notifs = await this.client.getNotifications(false);
        this.renderDropdown(notifs);
      } catch {
        this.dropdown.innerHTML = '<div style="padding:10px;color:#ff5555;">Failed to load</div>';
      }
    } else {
      this.dropdown.style.display = 'none';
    }
  }

  private renderDropdown(notifs: Notification[]): void {
    if (notifs.length === 0) {
      this.dropdown.innerHTML = '<div style="padding:10px;color:#555;text-align:center;">No notifications</div>';
      return;
    }

    const header = `<div style="padding:6px 10px;display:flex;justify-content:space-between;border-bottom:1px solid #333;">
      <span style="color:#ffff55;">Notifications</span>
      <button id="mark-all-read" style="font-family:inherit;background:none;border:none;color:#55ff55;cursor:pointer;font-size:11px;">Mark all read</button>
    </div>`;

    const items = notifs.slice(0, 20).map(n => {
      const icon = this.eventIcon(n.event_type);
      const readStyle = n.read ? 'color:#555;' : 'color:#aaa;';
      const bg = n.read ? '' : 'background:#0a1a0a;';
      return `<div class="notif-item" data-id="${n.id}" style="padding:8px 10px;border-bottom:1px solid #222;${bg}${readStyle}cursor:pointer;">
        <div style="display:flex;gap:6px;">
          <span>${icon}</span>
          <span style="flex:1;">${n.message}</span>
        </div>
        <div style="font-size:10px;color:#555;margin-top:2px;">${new Date(n.created_at).toLocaleString()}</div>
      </div>`;
    }).join('');

    this.dropdown.innerHTML = header + items;

    document.getElementById('mark-all-read')?.addEventListener('click', async () => {
      await this.client.markAllNotificationsRead();
      this.refresh();
    });

    this.dropdown.querySelectorAll('.notif-item').forEach(el => {
      el.addEventListener('click', async () => {
        const id = (el as HTMLElement).dataset.id!;
        await this.client.markNotificationRead(id);
        this.refresh();
      });
    });
  }

  private eventIcon(type: string): string {
    switch (type) {
      case 'your_turn': return '⚔';
      case 'game_started': return '🎮';
      case 'game_invite': return '📨';
      case 'under_attack': return '⚠';
      case 'turn_advanced': return '⏩';
      case 'player_joined': return '👤';
      case 'game_completed': return '🏁';
      default: return '📢';
    }
  }

  destroy(): void {
    if (this.pollTimer) clearInterval(this.pollTimer);
    this.container.remove();
  }
}
