-- Conquer DB Schema — Phase 3 (T094-T102)
-- Initial schema for Postgres persistence

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- Users (T099)
-- ============================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_admin BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_email ON users (email);

-- ============================================================
-- Games (T094)
-- ============================================================
CREATE TABLE games (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    seed BIGINT NOT NULL DEFAULT 42,
    status TEXT NOT NULL DEFAULT 'waiting_for_players'
        CHECK (status IN ('waiting_for_players', 'active', 'paused', 'completed')),
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_games_status ON games (status);

-- ============================================================
-- Game Worlds — per-turn world state snapshots (T095)
-- ============================================================
CREATE TABLE game_worlds (
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    turn INT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (game_id, turn)
);

-- ============================================================
-- Game Nations — per-turn nation state snapshots (T096)
-- ============================================================
CREATE TABLE game_nations (
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    nation_id INT NOT NULL,
    turn INT NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (game_id, nation_id, turn)
);

-- ============================================================
-- Game Sectors — per-turn sector grid snapshot (T097)
-- ============================================================
CREATE TABLE game_sectors (
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    turn INT NOT NULL,
    sector_data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (game_id, turn)
);

-- ============================================================
-- Game Actions — player-submitted actions (T098)
-- ============================================================
CREATE TABLE game_actions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    nation_id INT NOT NULL,
    turn INT NOT NULL,
    action JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    action_order INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_game_actions_game_turn ON game_actions (game_id, turn);
CREATE INDEX idx_game_actions_nation ON game_actions (game_id, nation_id, turn);

-- ============================================================
-- Game Players — user ↔ game mapping (T100)
-- ============================================================
CREATE TABLE game_players (
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nation_id INT NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_done_this_turn BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (game_id, user_id)
);

CREATE INDEX idx_game_players_user ON game_players (user_id);

-- ============================================================
-- Chat Messages (T101)
-- ============================================================
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    sender_nation_id INT,  -- NULL for system messages
    channel TEXT NOT NULL DEFAULT 'public',
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_game_channel ON chat_messages (game_id, channel);
CREATE INDEX idx_chat_created ON chat_messages (game_id, created_at DESC);

-- ============================================================
-- Game Invites (T102)
-- ============================================================
CREATE TABLE game_invites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    invite_code TEXT NOT NULL UNIQUE,
    created_by UUID NOT NULL REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    max_uses INT,
    uses INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_invites_code ON game_invites (invite_code);
