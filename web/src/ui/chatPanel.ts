// chatPanel.ts — Chat panel component for Phase 5 (T400-T408)
// Side panel / toggleable drawer with terminal aesthetic

import { GameClient } from '../network/client';
import { GameState } from '../state/gameState';
import { ChatMessageData, PublicNationInfo, DIPLO_NAMES } from '../types';
import { nationFgColor } from '../renderer/colors';

const CHAT_WIDTH = 340;
const MAX_MESSAGES = 200;

export class ChatPanel {
  private container: HTMLDivElement;
  private messageList: HTMLDivElement;
  private inputField: HTMLInputElement;
  private channelSelect: HTMLSelectElement;
  private unreadBadge: HTMLSpanElement;
  private presenceBar: HTMLDivElement;
  private client: GameClient;
  private state: GameState;
  private autoScroll = true;

  constructor(parent: HTMLElement, client: GameClient, state: GameState) {
    this.client = client;
    this.state = state;

    // Main container — right side drawer
    this.container = document.createElement('div');
    this.container.id = 'chat-panel';
    this.container.style.cssText = `
      position: fixed; right: 0; top: 0; bottom: 0;
      width: ${CHAT_WIDTH}px;
      background: #0a0a0a;
      border-left: 1px solid #333;
      display: flex; flex-direction: column;
      font-family: "Courier New", "Consolas", "Liberation Mono", monospace;
      font-size: 13px;
      color: #aaa;
      z-index: 100;
      transition: transform 0.2s ease;
    `;

    // Header with channel switcher + toggle
    const header = document.createElement('div');
    header.style.cssText = `
      display: flex; align-items: center; gap: 6px;
      padding: 6px 8px;
      background: #111;
      border-bottom: 1px solid #333;
      flex-shrink: 0;
    `;

    const title = document.createElement('span');
    title.textContent = '💬 CHAT';
    title.style.cssText = 'color: #55ff55; font-weight: bold; font-size: 12px;';

    this.channelSelect = document.createElement('select');
    this.channelSelect.style.cssText = `
      background: #1a1a1a; color: #aaa; border: 1px solid #444;
      font-family: inherit; font-size: 12px; padding: 2px 4px;
      flex: 1; cursor: pointer; outline: none;
    `;
    this.channelSelect.addEventListener('change', () => this.switchChannel(this.channelSelect.value));

    this.unreadBadge = document.createElement('span');
    this.unreadBadge.style.cssText = `
      background: #ff5555; color: #000; font-size: 10px; font-weight: bold;
      padding: 1px 5px; border-radius: 8px; display: none; min-width: 14px; text-align: center;
    `;

    const closeBtn = document.createElement('button');
    closeBtn.textContent = '✕';
    closeBtn.style.cssText = `
      background: none; border: none; color: #666; cursor: pointer;
      font-size: 16px; padding: 0 4px; font-family: inherit;
    `;
    closeBtn.addEventListener('click', () => this.toggle());

    header.appendChild(title);
    header.appendChild(this.channelSelect);
    header.appendChild(this.unreadBadge);
    header.appendChild(closeBtn);

    // Presence bar (T405)
    this.presenceBar = document.createElement('div');
    this.presenceBar.style.cssText = `
      padding: 3px 8px; background: #0d0d0d;
      border-bottom: 1px solid #222; font-size: 11px;
      color: #666; flex-shrink: 0; white-space: nowrap; overflow: hidden;
    `;
    this.presenceBar.textContent = 'Players: ...';

    // Message list (T401)
    this.messageList = document.createElement('div');
    this.messageList.style.cssText = `
      flex: 1; overflow-y: auto; padding: 4px 8px;
      scroll-behavior: smooth;
    `;
    this.messageList.addEventListener('scroll', () => {
      const el = this.messageList;
      // Auto-scroll detection
      this.autoScroll = (el.scrollHeight - el.scrollTop - el.clientHeight) < 40;
      // Load more on scroll to top (T401)
      if (el.scrollTop === 0) {
        this.loadMoreHistory();
      }
    });

    // Input area (T402)
    this.inputField = document.createElement('input');
    this.inputField.type = 'text';
    this.inputField.placeholder = 'Type a message... (/ for commands)';
    this.inputField.maxLength = 500;
    this.inputField.style.cssText = `
      background: #111; color: #55ff55; border: 1px solid #333;
      border-radius: 0; padding: 8px;
      font-family: inherit; font-size: 13px;
      outline: none; flex-shrink: 0;
      caret-color: #55ff55;
    `;
    this.inputField.addEventListener('keydown', (e) => {
      e.stopPropagation(); // Prevent game input handler from capturing
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        this.sendMessage();
      }
      if (e.key === 'Escape') {
        this.inputField.blur();
      }
    });

    this.container.appendChild(header);
    this.container.appendChild(this.presenceBar);
    this.container.appendChild(this.messageList);
    this.container.appendChild(this.inputField);
    parent.appendChild(this.container);

    // Start hidden
    this.setVisible(state.chatOpen);
    this.updateChannelList();
  }

  // ============ Public API ============

  toggle(): void {
    this.state.chatOpen = !this.state.chatOpen;
    this.setVisible(this.state.chatOpen);
    if (this.state.chatOpen) {
      // Clear unread for current channel
      this.state.chatUnread[this.state.chatChannel] = 0;
      this.updateUnreadBadge();
      this.scrollToBottom();
      setTimeout(() => this.inputField.focus(), 100);
    }
  }

  isOpen(): boolean {
    return this.state.chatOpen;
  }

  /** Handle incoming chat message from WebSocket */
  onChatMessage(msg: ChatMessageData): void {
    const channel = msg.channel;
    if (!this.state.chatMessages[channel]) {
      this.state.chatMessages[channel] = [];
    }
    this.state.chatMessages[channel].push(msg);

    // Trim to max
    if (this.state.chatMessages[channel].length > MAX_MESSAGES) {
      this.state.chatMessages[channel] = this.state.chatMessages[channel].slice(-MAX_MESSAGES);
    }

    // Update unread if not viewing this channel or panel closed
    if (!this.state.chatOpen || channel !== this.state.chatChannel) {
      this.state.chatUnread[channel] = (this.state.chatUnread[channel] || 0) + 1;
      this.updateUnreadBadge();
    }

    // Render if viewing this channel
    if (channel === this.state.chatChannel) {
      this.appendMessageElement(msg);
      if (this.autoScroll) this.scrollToBottom();
    }

    // Add to channel list if new
    if (!this.state.chatChannels.includes(channel)) {
      this.state.chatChannels.push(channel);
      this.updateChannelList();
    }
  }

  /** Handle chat history response */
  onChatHistory(channel: string, messages: ChatMessageData[]): void {
    if (!this.state.chatMessages[channel]) {
      this.state.chatMessages[channel] = [];
    }
    // Prepend (history is older messages)
    const existing = this.state.chatMessages[channel];
    const existingTimestamps = new Set(existing.map(m => m.timestamp));
    const newMsgs = messages.filter(m => !existingTimestamps.has(m.timestamp));
    this.state.chatMessages[channel] = [...newMsgs.reverse(), ...existing];

    if (channel === this.state.chatChannel) {
      this.renderMessages();
    }
  }

  /** Handle presence update */
  onPresenceUpdate(nationId: number, status: string): void {
    if (status === 'online') {
      this.state.onlineNations.add(nationId);
    } else {
      this.state.onlineNations.delete(nationId);
    }
    this.updatePresenceBar();
  }

  /** Set initial presence list */
  setPresence(nationIds: number[]): void {
    this.state.onlineNations = new Set(nationIds);
    this.updatePresenceBar();
  }

  /** Update the public nations list (for channel names) */
  updateNations(): void {
    this.updateChannelList();
    this.updatePresenceBar();
  }

  destroy(): void {
    this.container.remove();
  }

  // ============ Internal ============

  private setVisible(visible: boolean): void {
    this.container.style.transform = visible ? 'translateX(0)' : `translateX(${CHAT_WIDTH}px)`;
  }

  private switchChannel(channel: string): void {
    this.state.chatChannel = channel;
    this.state.chatUnread[channel] = 0;
    this.updateUnreadBadge();
    this.renderMessages();

    // Request history if we have no messages for this channel
    if (!this.state.chatMessages[channel] || this.state.chatMessages[channel].length === 0) {
      if (this.state.gameId) {
        this.client.requestChatHistory(channel);
      }
    }
    this.scrollToBottom();
  }

  private updateChannelList(): void {
    const nations = this.state.publicNations;
    this.channelSelect.innerHTML = '';

    for (const ch of this.state.chatChannels) {
      const opt = document.createElement('option');
      opt.value = ch;
      if (ch === 'public') {
        opt.textContent = '📢 Public';
      } else {
        opt.textContent = this.formatChannelName(ch, nations);
      }
      if (ch === this.state.chatChannel) opt.selected = true;
      this.channelSelect.appendChild(opt);
    }

    // Add option to start new private channel
    if (nations.length > 0 && this.state.nationId) {
      const sep = document.createElement('option');
      sep.disabled = true;
      sep.textContent = '── New Private ──';
      this.channelSelect.appendChild(sep);

      for (const n of nations) {
        if (n.nation_id === this.state.nationId) continue;
        const chName = this.privateChannelName(this.state.nationId, n.nation_id);
        if (this.state.chatChannels.includes(chName)) continue;
        const opt = document.createElement('option');
        opt.value = chName;
        opt.textContent = `🔒 → ${n.name}`;
        this.channelSelect.appendChild(opt);
      }
    }
  }

  private formatChannelName(channel: string, nations: PublicNationInfo[]): string {
    const parts = channel.split('_');
    if (parts.length === 3 && parts[0] === 'nation') {
      const a = parseInt(parts[1]);
      const b = parseInt(parts[2]);
      const other = (a === this.state.nationId) ? b : a;
      const nation = nations.find(n => n.nation_id === other);
      return `🔒 ${nation?.name ?? `Nation ${other}`}`;
    }
    return channel;
  }

  private privateChannelName(a: number, b: number): string {
    const lo = Math.min(a, b);
    const hi = Math.max(a, b);
    return `nation_${lo}_${hi}`;
  }

  private sendMessage(): void {
    const text = this.inputField.value.trim();
    if (!text) return;
    this.inputField.value = '';

    // Handle slash commands (T408)
    if (text.startsWith('/')) {
      this.handleSlashCommand(text);
      return;
    }

    this.client.sendChatMessage(this.state.chatChannel, text);

    // If channel is new, add to list
    if (!this.state.chatChannels.includes(this.state.chatChannel)) {
      this.state.chatChannels.push(this.state.chatChannel);
      this.updateChannelList();
    }
  }

  /** Handle slash commands (T408) */
  private handleSlashCommand(text: string): void {
    const parts = text.split(/\s+/);
    const cmd = parts[0].toLowerCase();

    let output = '';
    switch (cmd) {
      case '/who': {
        const nations = this.state.publicNations;
        if (nations.length === 0) {
          output = 'No nations in game.';
        } else {
          const lines = nations.map(n => {
            const online = this.state.onlineNations.has(n.nation_id) ? '●' : '○';
            return `${online} ${n.name} [${n.race}] Score:${n.score}`;
          });
          output = lines.join('\n');
        }
        break;
      }
      case '/diplo': {
        if (!this.state.nation) {
          output = 'No nation data loaded.';
        } else {
          const lines = this.state.publicNations
            .filter(n => n.nation_id !== this.state.nationId)
            .map(n => {
              const status = this.state.nation!.diplomacy[n.nation_id] ?? 0;
              return `${n.name}: ${DIPLO_NAMES[status] ?? 'UNKNOWN'}`;
            });
          output = lines.length > 0 ? lines.join('\n') : 'No diplomacy data.';
        }
        break;
      }
      case '/score': {
        const nations = [...this.state.publicNations].sort((a, b) => b.score - a.score);
        const lines = nations.map((n, i) => `${i + 1}. ${n.name}: ${n.score}`);
        output = lines.length > 0 ? lines.join('\n') : 'No scores.';
        break;
      }
      case '/help':
        output = 'Commands: /who — list players, /diplo — diplomacy, /score — scoreboard, /help — this message';
        break;
      default:
        output = `Unknown command: ${cmd}. Try /help`;
    }

    // Display locally as system message
    const sysMsg: ChatMessageData = {
      sender_nation_id: null,
      sender_name: 'SYSTEM',
      channel: this.state.chatChannel,
      content: output,
      timestamp: new Date().toISOString(),
      is_system: true,
    };
    this.onChatMessage(sysMsg);
  }

  private renderMessages(): void {
    this.messageList.innerHTML = '';
    const msgs = this.state.chatMessages[this.state.chatChannel] ?? [];
    for (const msg of msgs) {
      this.appendMessageElement(msg);
    }
  }

  private appendMessageElement(msg: ChatMessageData): void {
    const el = document.createElement('div');
    el.style.cssText = 'padding: 2px 0; word-wrap: break-word; line-height: 1.4;';

    if (msg.is_system) {
      // System message styling (T407)
      el.style.color = '#888';
      el.style.fontStyle = 'italic';
      el.style.borderLeft = '2px solid #444';
      el.style.paddingLeft = '6px';
      el.style.marginLeft = '2px';
      // Preserve newlines for /command output
      el.style.whiteSpace = 'pre-wrap';
      el.textContent = msg.content;
    } else {
      // Player message (T406)
      const time = this.formatTime(msg.timestamp);
      const timeSpan = document.createElement('span');
      timeSpan.style.cssText = 'color: #555; font-size: 11px;';
      timeSpan.textContent = time + ' ';

      const nameSpan = document.createElement('span');
      nameSpan.style.cssText = `font-weight: bold; color: ${this.getNameColor(msg.sender_nation_id)};`;
      // Show short name (nation name only, not "(Leader)")
      const shortName = msg.sender_name.split(' (')[0];
      nameSpan.textContent = shortName + ': ';

      const textSpan = document.createElement('span');
      textSpan.style.color = '#ccc';
      textSpan.textContent = msg.content;

      el.appendChild(timeSpan);
      el.appendChild(nameSpan);
      el.appendChild(textSpan);
    }

    this.messageList.appendChild(el);
  }

  private formatTime(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  }

  private getNameColor(nationId: number | null): string {
    if (nationId === null) return '#888';
    return nationFgColor(nationId);
  }

  private scrollToBottom(): void {
    requestAnimationFrame(() => {
      this.messageList.scrollTop = this.messageList.scrollHeight;
    });
  }

  private loadMoreHistory(): void {
    const msgs = this.state.chatMessages[this.state.chatChannel];
    if (!msgs || msgs.length === 0) return;
    const oldest = msgs[0];
    this.client.requestChatHistory(this.state.chatChannel, oldest.timestamp, 50);
  }

  private updateUnreadBadge(): void {
    const total = Object.values(this.state.chatUnread).reduce((sum, n) => sum + n, 0);
    if (total > 0) {
      this.unreadBadge.textContent = String(total);
      this.unreadBadge.style.display = 'inline';
    } else {
      this.unreadBadge.style.display = 'none';
    }
  }

  private updatePresenceBar(): void {
    const nations = this.state.publicNations;
    if (nations.length === 0) {
      this.presenceBar.textContent = 'No players';
      return;
    }

    const parts: string[] = [];
    for (const n of nations) {
      const online = this.state.onlineNations.has(n.nation_id);
      const dot = online ? '🟢' : '⚫';
      parts.push(`${dot}${n.name}`);
    }
    this.presenceBar.textContent = parts.join('  ');
  }
}
