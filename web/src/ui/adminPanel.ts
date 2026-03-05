// adminPanel.ts — Admin dashboard for game creators (T423-T427)

import { GameClient } from '../network/client';
import { AdminPlayerInfo, TurnSnapshotInfo, GameInfo } from '../types';

export class AdminPanel {
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
      const [game, players, snapshots] = await Promise.all([
        this.client.getGame(this.gameId),
        this.client.adminListPlayers(this.gameId),
        this.client.adminListSnapshots(this.gameId),
      ]);
      this.render(game, players, snapshots);
    } catch (e) {
      this.container.innerHTML = `
        <p style="color:#ff5555;">Admin access denied or error: ${(e as Error).message}</p>
        <button id="admin-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Back</button>
      `;
      document.getElementById('admin-back')!.addEventListener('click', this.onClose);
    }
  }

  private render(game: GameInfo, players: AdminPlayerInfo[], snapshots: TurnSnapshotInfo[]): void {
    const playerRows = players.map(p => `
      <tr>
        <td style="padding:3px 8px;color:#55ffff;">${p.nation_name}</td>
        <td style="padding:3px 8px;">${p.nation_id}</td>
        <td style="padding:3px 8px;">${p.race}</td>
        <td style="padding:3px 8px;text-align:right;">${p.score}</td>
        <td style="padding:3px 8px;">${p.is_done ? '<span style="color:#55ff55;">Done</span>' : '<span style="color:#ffff55;">Waiting</span>'}</td>
        <td style="padding:3px 8px;"><button class="kick-btn" data-nid="${p.nation_id}" style="font-family:inherit;background:#440000;color:#ff5555;border:1px solid #ff5555;padding:2px 8px;cursor:pointer;font-size:12px;">Kick</button></td>
      </tr>
    `).join('');

    const snapshotRows = snapshots.map(s => `
      <tr>
        <td style="padding:3px 8px;">Turn ${s.turn}</td>
        <td style="padding:3px 8px;">${new Date(s.created_at).toLocaleString()}</td>
        <td style="padding:3px 8px;"><button class="rollback-btn" data-turn="${s.turn}" style="font-family:inherit;background:#444400;color:#ffff55;border:1px solid #ffff55;padding:2px 8px;cursor:pointer;font-size:12px;">Rollback</button></td>
      </tr>
    `).join('');

    const statusColor = game.status === 'active' ? '#55ff55' : game.status === 'paused' ? '#ffff55' : '#aaa';

    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">⚙ ADMIN: ${game.name}</h1>
      <button id="admin-back" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;position:absolute;top:10px;left:10px;">← Back</button>

      <div style="max-width:700px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Game Status</h2>
        <p>Status: <span style="color:${statusColor};">${game.status}</span> | Turn: ${game.current_turn} | Players: ${game.player_count}</p>
        <div style="display:flex;gap:8px;flex-wrap:wrap;">
          ${game.status !== 'paused' ? '<button id="btn-pause" style="font-family:inherit;background:#222;color:#ffff55;border:1px solid #ffff55;padding:6px 14px;cursor:pointer;">⏸ Pause</button>' : ''}
          ${game.status === 'paused' ? '<button id="btn-resume" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:6px 14px;cursor:pointer;">▶ Resume</button>' : ''}
          <button id="btn-advance" style="font-family:inherit;background:#222;color:#55ffff;border:1px solid #55ffff;padding:6px 14px;cursor:pointer;">⏩ Advance Turn</button>
          <button id="btn-complete" style="font-family:inherit;background:#222;color:#ff5555;border:1px solid #ff5555;padding:6px 14px;cursor:pointer;">🏁 End Game</button>
        </div>
        <div id="status-msg" style="margin:6px 0;"></div>
      </div>

      <div style="max-width:700px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Players</h2>
        <table style="width:100%;border-collapse:collapse;">
          <tr style="border-bottom:1px solid #333;">
            <th style="text-align:left;padding:3px 8px;color:#aaa;">Nation</th>
            <th style="text-align:left;padding:3px 8px;color:#aaa;">ID</th>
            <th style="text-align:left;padding:3px 8px;color:#aaa;">Race</th>
            <th style="text-align:right;padding:3px 8px;color:#aaa;">Score</th>
            <th style="text-align:left;padding:3px 8px;color:#aaa;">Status</th>
            <th style="padding:3px 8px;"></th>
          </tr>
          ${playerRows || '<tr><td colspan="6" style="color:#555;padding:8px;">No players yet.</td></tr>'}
        </table>
      </div>

      <div style="max-width:700px;width:100%;margin:20px 0;">
        <h2 style="color:#ffff55;">Turn Rollback</h2>
        ${snapshotRows
          ? `<table style="width:100%;border-collapse:collapse;">
              <tr style="border-bottom:1px solid #333;">
                <th style="text-align:left;padding:3px 8px;color:#aaa;">Turn</th>
                <th style="text-align:left;padding:3px 8px;color:#aaa;">Saved</th>
                <th style="padding:3px 8px;"></th>
              </tr>
              ${snapshotRows}
            </table>`
          : '<p style="color:#555;">No snapshots available yet. Snapshots are created each turn advance.</p>'
        }
      </div>
    `;

    // Event handlers
    document.getElementById('admin-back')!.addEventListener('click', this.onClose);
    document.getElementById('btn-pause')?.addEventListener('click', () => this.setStatus('paused'));
    document.getElementById('btn-resume')?.addEventListener('click', () => this.setStatus('active'));
    document.getElementById('btn-advance')!.addEventListener('click', () => this.advanceTurn());
    document.getElementById('btn-complete')!.addEventListener('click', () => this.setStatus('completed'));

    this.container.querySelectorAll('.kick-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const nid = parseInt((e.target as HTMLElement).dataset.nid!);
        if (confirm(`Kick nation ${nid}? This cannot be undone.`)) {
          this.kickPlayer(nid);
        }
      });
    });

    this.container.querySelectorAll('.rollback-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const turn = parseInt((e.target as HTMLElement).dataset.turn!);
        if (confirm(`Rollback to turn ${turn}? All progress after will be lost.`)) {
          this.rollback(turn);
        }
      });
    });
  }

  private async setStatus(status: string): Promise<void> {
    const msg = document.getElementById('status-msg')!;
    try {
      await this.client.adminSetStatus(this.gameId, status);
      msg.style.color = '#55ff55';
      msg.textContent = `Status set to ${status}`;
      setTimeout(() => this.load(), 500);
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  private async advanceTurn(): Promise<void> {
    const msg = document.getElementById('status-msg')!;
    try {
      await this.client.adminAdvanceTurn(this.gameId);
      msg.style.color = '#55ff55';
      msg.textContent = 'Turn advanced!';
      setTimeout(() => this.load(), 500);
    } catch (e) {
      msg.style.color = '#ff5555';
      msg.textContent = `Error: ${(e as Error).message}`;
    }
  }

  private async kickPlayer(nationId: number): Promise<void> {
    try {
      await this.client.adminKickPlayer(this.gameId, nationId);
      this.load();
    } catch (e) {
      alert(`Kick failed: ${(e as Error).message}`);
    }
  }

  private async rollback(turn: number): Promise<void> {
    try {
      await this.client.adminRollback(this.gameId, turn);
      this.load();
    } catch (e) {
      alert(`Rollback failed: ${(e as Error).message}`);
    }
  }

  destroy(): void {
    this.container.remove();
  }
}
