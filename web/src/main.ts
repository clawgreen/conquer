// main.ts — Entry point for Conquer web frontend

import { GameClient } from './network/client';
import { LobbyScreen } from './ui/lobby';
import { GameScreen } from './game/gameScreen';

// Global styles — terminal aesthetic
document.documentElement.style.cssText = `
  margin: 0; padding: 0;
  background: #000;
  overflow: hidden;
`;
document.body.style.cssText = `
  margin: 0; padding: 0;
  background: #000;
  color: #aaa;
  font-family: "Courier New", "Consolas", "Liberation Mono", monospace;
  overflow: hidden;
`;

const app = document.getElementById('app')!;
const client = new GameClient();

// Restore token if we have one
const savedToken = localStorage.getItem('conquer_token');
if (savedToken) {
  client.setToken(savedToken);
}

let currentScreen: LobbyScreen | GameScreen | null = null;

function showLobby(): void {
  if (currentScreen) {
    if (currentScreen instanceof GameScreen) currentScreen.destroy();
    if (currentScreen instanceof LobbyScreen) currentScreen.destroy();
  }
  app.innerHTML = '';

  currentScreen = new LobbyScreen(app, client, (gameId, nationId) => {
    showGame(gameId, nationId);
  });
}

function showGame(gameId: string, nationId: number): void {
  if (currentScreen) {
    if (currentScreen instanceof LobbyScreen) currentScreen.destroy();
    if (currentScreen instanceof GameScreen) currentScreen.destroy();
  }
  app.innerHTML = '';

  currentScreen = new GameScreen(app, client, gameId, nationId);
}

// Start with lobby
showLobby();
