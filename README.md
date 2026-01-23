# Kanitomo

A terminal pet crab that thrives on your git commits.

## Demo

![Kanitomo Demo](assets/preview.png)

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
- Happiness system that rewards consistent coding
- Weekends off - Kani won't get sad while you rest
- Persistent state - Kani remembers you between sessions

## Installation

```bash
cargo install --path .
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
| `q` | Quit |

### Reset Stats

Start fresh by clearing all stats (happiness, streak, commit history):

```bash
kanitomo --reset
```

## License

MIT
