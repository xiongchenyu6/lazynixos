package main

import (
	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
)

type KeyMap struct {
	Up       key.Binding
	Down     key.Binding
	Enter    key.Binding
	Build    key.Binding
	DryBuild key.Binding
	Tab      key.Binding
	Quit     key.Binding
	Help     key.Binding
}

var _ help.KeyMap = (*KeyMap)(nil)

// ShortHelp returns bindings shown in the mini help bar
func (k KeyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Enter, k.Build, k.DryBuild, k.Tab, k.Quit}
}

// FullHelp returns bindings shown in the expanded help view
func (k KeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Up, k.Down},
		{k.Enter, k.Build, k.DryBuild},
		{k.Tab, k.Help, k.Quit},
	}
}

var DefaultKeyMap = KeyMap{
	Up: key.NewBinding(
		key.WithKeys("up", "k"),
		key.WithHelp("↑/k", "up"),
	),
	Down: key.NewBinding(
		key.WithKeys("down", "j"),
		key.WithHelp("↓/j", "down"),
	),
	Enter: key.NewBinding(
		key.WithKeys("enter"),
		key.WithHelp("enter", "switch"),
	),
	Build: key.NewBinding(
		key.WithKeys("b"),
		key.WithHelp("b", "build"),
	),
	DryBuild: key.NewBinding(
		key.WithKeys("d"),
		key.WithHelp("d", "dry-build"),
	),
	Tab: key.NewBinding(
		key.WithKeys("tab"),
		key.WithHelp("tab", "toggle pane"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
	Help: key.NewBinding(
		key.WithKeys("?"),
		key.WithHelp("?", "help"),
	),
}
