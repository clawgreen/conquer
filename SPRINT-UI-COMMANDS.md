# Sprint: Wire All Commands to Web UI

## Context
The Rust engine now has full command parity with the C original. All Action variants exist
and are handled in apply_action_to_state. This sprint wires them into the web UI so players 
can actually USE them.

## Architecture
- Commands send Actions via `client.submitActions(gameId, [{ ActionName: { ...params } }])`
- Keyboard shortcuts defined in `src/game/inputHandler.ts` (GameAction types)
- Command sidebar buttons in `src/ui/commandSidebar.ts` (CmdDef groups)
- Game screen handles actions in `src/game/gameScreen.ts` (handleCommand/handleAction)
- Modal dialogs for complex inputs via `src/ui/modalDialog.ts`

## Rules
- Read existing code patterns before adding new commands
- Use modalDialog.ts for any command that needs parameters (which army, how many soldiers, etc.)
- Add keyboard shortcuts following C conventions where possible
- Add sidebar buttons in appropriate groups
- All actions go through client.submitActions()
- Run `npx tsc --noEmit` after each change to verify
- Commit after each task

## Reference: C Keyboard Commands (from original help screen)
```
MOVEMENT:        Arrow keys, hjkl, yubn (vi-style diagonals)
DISPLAY MODES:   d=designation, c=contour, v=vegetation, f=food, r=race, n=nation
                 p=people, m=movement, i=items, J=jewels/gold, M=defense, D=people
HIGHLIGHT:       o=own, a=armies, y=yours, s=trade, x=none, L=move range
ARMY COMMANDS:   Tab=next army, m=move mode, Space=done moving
                 +=combine, -=split, /=divide
                 a=attack, d=defend, g=garrison, s=scout, R=rule, S=march(sent)
SECTOR:          R=redesignate, P=draft troops, F=build fort, W=build road
NATION:          E=end turn, S=scores, N=news, B=budget/spreadsheet
                 X=diplomacy, G=goto capitol, ?=help
MAGIC:           Z=cast spell, Q=buy power
NAVY:            `=toggle army/navy, L=load fleet, U=unload fleet
TRADE:           $=propose trade, T=toggle chat
MISC:            +=font up, -=font down, C=center
```

## Tasks

### T1: Army Status Commands (Attack/Defend/Garrison/Scout/Rule/March)
- Add GameAction types: `set_army_attack`, `set_army_defend`, `set_army_garrison`, 
  `set_army_scout`, `set_army_rule`, `set_army_march`
- In gameScreen.ts, when an army is selected, these submit AdjustArmyStat actions
- Add keyboard shortcuts in inputHandler.ts (only active when army selected):
  - After selecting army, press: a=attack, d=defend, g=garrison, s=scout, R=rule, S=march
- Add to command sidebar Army group
- Show current status in army info display

### T2: Army Split (-)
- Add `split_army` GameAction
- When army selected and `-` pressed, show modal: "Split how many soldiers?"
- Default: half. Input field for custom amount.
- Submit SplitArmy action
- Add button to Army group in sidebar

### T3: Army Combine (+)
- Add `combine_army` GameAction
- When army selected and `+` pressed:
  - Find other armies at same location
  - If only one other, combine automatically
  - If multiple, show modal to pick which army to combine with
- Submit CombineArmies action
- Add button to Army group in sidebar

### T4: Army Divide (/)
- Add `divide_army` GameAction
- When army selected and `/` pressed, divide equally (no modal needed)
- Submit DivideArmy action
- Add button to Army group in sidebar

### T5: Redesignate Sector (R)
- Currently `redesignate` GameAction exists but may not be fully wired
- When R pressed at cursor location on owned sector, show modal with valid designations:
  Capitol, City, Town, Farm, Mine, Gold Mine, Fort, Port, Road, Ruin, etc.
- Submit DesignateSector action
- Show designation names/descriptions in modal

### T6: Draft/Enlist Troops (P)
- Currently `draft` GameAction exists but may not be wired  
- When P pressed on owned sector with people, show modal:
  - Unit type dropdown (valid for your race)
  - Number to draft (from sector population)
  - Show cost in gold
- Submit DraftUnit action
- Add to Actions group in sidebar

### T7: Build Fortification (F)
- Add `build_fort` GameAction and key binding (F)
- When F pressed on owned sector with city/town/fort:
  - Show current fort level and cost for next level
  - Confirm dialog
- Submit ConstructFort action
- Add to Actions group

### T8: Build Road (W)
- Add `build_road` GameAction and key binding (W)
- When W pressed on owned sector:
  - Show cost, confirm
  - Track roads-per-turn limit
- Submit BuildRoad action
- Add to Actions group

### T9: Build Ships
- Add `build_ship` GameAction
- Show modal when in coastal city:
  - Ship type: Warship, Merchant, Galley
  - Size: Small, Medium, Large
  - Count
  - Show cost
- Submit ConstructShip action
- Add to Actions/Navy group

### T10: Navy Operations — Load/Unload
- Add navy commands to sidebar and keybindings:
  - `L` (when navy selected) = Load army onto fleet  
  - `U` (when navy selected) = Unload army from fleet
- Show modal to pick which army (must be at same location)
- Submit LoadArmyOnFleet / UnloadArmyFromFleet actions
- Also support LoadPeopleOnFleet / UnloadPeople for civilian transport

### T11: Magic — Cast Spell (Z)
- Add `cast_spell` GameAction and Z keybinding
- Show modal with available spells (based on nation.powers):
  - Summon creature (pick type)
  - Flight (pick army)  
  - Attack Enhancement
  - Defense Enhancement
  - Destroy (pick target sector)
- Show spell point cost for each
- Submit CastSpell action
- Add to new "Magic" command group in sidebar

### T12: Magic — Buy Power (Q)
- Add `buy_power` GameAction and Q keybinding
- Show modal listing available powers to purchase:
  - Name, cost in gold, description
  - Grayed out if can't afford or already owned
- Submit BuyMagicPower action
- Add to Magic group in sidebar

### T13: Diplomacy Screen (X)
- Add `diplomacy` GameAction and X keybinding (may already exist)
- Show modal/panel with all known nations:
  - Current diplomatic status (Allied → Friendly → Neutral → Hostile → War)
  - Buttons to change (one step per turn)
  - Race and score info
- Submit AdjustDiplomacy action
- Add to Actions group in sidebar

### T14: Trade — Propose ($)
- Add `propose_trade` GameAction and `$` keybinding
- Show modal:
  - Target nation (dropdown)
  - Offer: type (gold/food/metal/jewels/army/land) + amount
  - Request: type + amount
- Submit ProposeTrade action
- Add to new "Trade" command group

### T15: Trade — Accept/Reject Incoming
- Add trade notification panel or modal
- When trade proposals exist, show:
  - Who proposed what
  - Accept / Reject buttons
- Submit AcceptTrade / RejectTrade actions
- Show pending trades count in HUD

### T16: Hire Mercenaries
- Add `hire_mercs` GameAction
- Show modal:
  - Available mercenaries (world.merc_men)
  - Attack/defense stats
  - Cost per soldier
  - How many to hire
- Submit HireMercenaries action
- Add to Actions group

### T17: Bribery
- Add `bribe` GameAction
- Show modal:
  - Target nation (dropdown)
  - Amount of gold to spend
  - Show success probability
- Submit BribeNation action
- Add to Diplomacy section or Actions group

### T18: Send Tribute
- Add `send_tribute` GameAction
- Show modal:
  - Target nation
  - Resources to send (gold, food, metal, jewels sliders/inputs)
- Submit SendTribute action
- Add to Trade or Diplomacy group

### T19: Budget/Spreadsheet Report (B)
- `show_budget` GameAction may already exist
- Show modal/panel with full nation economy report:
  - Income by sector type
  - Military maintenance costs
  - Treasury, food, metal, jewels
  - Population growth/decline
  - Fetch from server spreadsheet endpoint or compute client-side
- Make it look like the C terminal spreadsheet output

### T20: Help Screen (?)
- `show_help` GameAction exists
- Show modal with complete command reference:
  - All keyboard shortcuts organized by category
  - Match the reference table from this file's header
- Include game rules summary

### T21: Sector Info Panel
- When cursor is on a sector, show detailed info:
  - Owner, designation, population, fortress level
  - Terrain, vegetation, altitude
  - Resources (food production, metal, jewels, tradegoods)
  - Armies present
- Update in real-time as cursor moves
- This may partially exist in statsSidebar — enhance it

### T22: Complete Keybindings Modal
- Update keybindingsModal.ts to include ALL new keybindings
- Make keybindings remappable (store in localStorage)
- Group by category matching the command reference
- Show current binding + default

### T23: Mobile Command Menu
- Update mobileToolbar.ts or touchHandler.ts
- Add touch-friendly command menus for all new commands
- Long-press on army → context menu (split, combine, status, etc.)
- Long-press on sector → context menu (redesignate, draft, build, etc.)
- Swipe gestures for common actions
