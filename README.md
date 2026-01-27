# Kanitomo

A terminal pet crab that thrives on your git commits.

## Demo

https://github.com/user-attachments/assets/8840a134-07af-4b66-a437-713135699078

*Meet Kani, your new coding companion. Feed them commits and watch them thrive!*

## About the Name

The name "Kanitomo" is inspired by [Tamagotchi](https://en.wikipedia.org/wiki/Tamagotchi), the famous virtual pet from the 90s. "Tamagotchi" is a portmanteau of two Japanese words:
- **tamago** (卵) - egg
- **tomodachi** (友達) - friend

Following this pattern, "Kanitomo" combines:
- **kani** (蟹) - crab
- **tomo** (友) - friend

So "Kanitomo" literally means "crab friend" - your little terminal companion that keeps you company while you code.

## Features

- Watch Kani react to your git commits in real-time
- **Kani talks!** - Dynamic mood-based messages in the title bar
- **Multi-repo support** - Run in your dev folder to watch all projects at once
- **Commit tracking** - See today's commits per project and weekly summary
- **Mini-games** - Take a break with Crab Catch, Snake, or Breakout and chase high scores
- Happiness is driven by today's commits (fast early gains, slower near the top)
- Weekends off - Kani won't get sad while you rest
- Persistent state - Kani remembers you between sessions

## Installation

```bash
cargo install --path .
```

Or install directly from GitHub:

```bash
cargo install --git https://github.com/YannickHerrero/kanitomo
```

## Usage

Run `kanitomo` in any git repository:

```bash
cd ~/projects/my-app
kanitomo
```

Or run it in a parent folder to watch multiple repositories at once:

```bash
cd ~/projects
kanitomo
```

Kani will automatically discover git repositories in immediate subdirectories and react to commits in any of them.

### Keybindings

| Key | Action |
|-----|--------|
| `a` | View watched repositories |
| `d` | View commit stats (today's commits by project, weekly summary) |
| `s` | Toggle stats panel |
| `space` | Open mini-game menu |
| `?` | Toggle help window |
| `q` | Quit |

### Mini Games

Press `space` to open the mini-game menu. Each game has its own leaderboard with top 100 scores tracked.

#### Crab Catch
Catch falling food with Kani in a 20-second timed round. The crab shows a happy face when catching food!

| Key | Action |
|-----|--------|
| `←` `→` / `h` `l` | Move crab left/right |
| `q` | Exit game |

#### Snake
Classic snake game - eat food to grow, avoid walls and yourself. Speed increases as you grow longer.

| Key | Action |
|-----|--------|
| `←` `→` `↑` `↓` / `h` `j` `k` `l` | Change direction |
| `q` | Exit game |

#### Breakout
Break all the bricks with a bouncing ball. You have 3 lives, and higher rows are worth more points (10-50 pts). Ball speed increases as you destroy bricks.

| Key | Action |
|-----|--------|
| `←` `→` / `h` `l` | Move paddle |
| `space` | Launch ball |
| `q` | Exit game |

### Happiness

Happiness is based solely on how many commits you make today. It rises quickly at first and slows near the top, reaching 100% at 20 commits.

### Debug Keybindings

Run with `--debug` to enable:

| Key | Action |
|-----|--------|
| `f` | Add a debug commit |
| `p` | Remove a debug commit |
| `g` | Cycle ground styles |
| `c` | Toggle fast cycle |
| `x` | Freeze movement |

### Reset Stats

Start fresh by clearing all stats (happiness, streak, commit history):

```bash
kanitomo --reset
```

## License

MIT
