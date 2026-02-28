package main

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type Model struct {
	list       list.Model
	viewport   viewport.Model
	help       help.Model
	keys       KeyMap
	logs       strings.Builder
	ready      bool   // true after first WindowSizeMsg
	running    bool   // true while a command is executing
	activeHost string // hostname currently being deployed
	activeCmd  string // "switch", "build", or "dry-build"
	lastErr    error  // last command error (nil = success)
	width      int
	height     int
	focusPane  int    // 0 = list, 1 = viewport
	flakePath  string // resolved flake directory path
}

func NewModel(flakePath string) Model {
	l := list.New([]list.Item{}, list.NewDefaultDelegate(), 0, 0)
	l.Title = "LazyNixOS"

	vp := viewport.New(0, 0)

	return Model{
		list:      l,
		viewport:  vp,
		help:      help.New(),
		keys:      DefaultKeyMap,
		flakePath: flakePath,
	}
}

func (m Model) Init() tea.Cmd {
	return FetchHosts(m.flakePath)
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd
	var cmd tea.Cmd

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

		leftWidth := int(float64(m.width) * LeftPaneRatio)
		rightWidth := m.width - leftWidth

		m.list.SetSize(leftWidth-2, m.height-3)
		m.viewport.Width = rightWidth - 4
		m.viewport.Height = m.height - 5
		m.ready = true

	case HostsLoadedMsg:
		items := make([]list.Item, len(msg.hosts))
		for i, h := range msg.hosts {
			items[i] = h
		}
		cmd = m.list.SetItems(items)
		cmds = append(cmds, cmd)

	case HostsErrorMsg:
		m.logs.WriteString(msg.err.Error() + "\n")
		m.viewport.SetContent(m.logs.String())

	case CommandStartedMsg:
		m.running = true
		m.activeHost = msg.Host
		m.activeCmd = msg.Action
		m.logs.Reset()
		m.logs.WriteString(fmt.Sprintf("Starting nixos-rebuild %s on %s...\n", msg.Action, msg.Host))
		m.viewport.SetContent(m.logs.String())

	case LogLineMsg:
		m.logs.WriteString(msg.Line + "\n")
		m.viewport.SetContent(m.logs.String())
		m.viewport.GotoBottom()

	case CommandFinishedMsg:
		m.running = false
		m.lastErr = msg.Err
		if msg.Err != nil {
			m.logs.WriteString(fmt.Sprintf("\nCommand failed: %v\n", msg.Err))
		} else {
			m.logs.WriteString("\nCommand completed successfully.\n")
		}
		m.viewport.SetContent(m.logs.String())
		m.viewport.GotoBottom()

	case tea.KeyMsg:
		if m.list.FilterState() == list.Filtering {
			m.list, cmd = m.list.Update(msg)
			cmds = append(cmds, cmd)
			return m, tea.Batch(cmds...)
		}

		switch {
		case key.Matches(msg, m.keys.Quit):
			return m, tea.Quit
		case key.Matches(msg, m.keys.Tab):
			if m.focusPane == 0 {
				m.focusPane = 1
			} else {
				m.focusPane = 0
			}
		case key.Matches(msg, m.keys.Enter), key.Matches(msg, m.keys.Build), key.Matches(msg, m.keys.DryBuild):
			if m.running {
				m.logs.WriteString("Command already running\n")
				m.viewport.SetContent(m.logs.String())
				m.viewport.GotoBottom()
				break
			}

			if i, ok := m.list.SelectedItem().(HostItem); ok {
				action := "switch"
				if key.Matches(msg, m.keys.Build) {
					action = "build"
				} else if key.Matches(msg, m.keys.DryBuild) {
					action = "dry-build"
				}
				cmds = append(cmds, RunNixosRebuild(m.flakePath, i.name, action))
			}
		}
	}

	if m.focusPane == 0 {
		m.list, cmd = m.list.Update(msg)
		cmds = append(cmds, cmd)
	} else {
		m.viewport, cmd = m.viewport.Update(msg)
		cmds = append(cmds, cmd)
	}

	return m, tea.Batch(cmds...)
}

func (m Model) View() string {
	if !m.ready {
		return "Initializing..."
	}

	leftWidth := int(float64(m.width) * LeftPaneRatio)
	rightWidth := m.width - leftWidth

	var leftPane, rightPane string

	if m.focusPane == 0 {
		leftPane = ActivePaneStyle.Width(leftWidth - 2).Height(m.height - 3).Render(m.list.View())
		rightPane = InactivePaneStyle.Width(rightWidth - 2).Height(m.height - 3).Render(m.viewport.View())
	} else {
		leftPane = InactivePaneStyle.Width(leftWidth - 2).Height(m.height - 3).Render(m.list.View())
		rightPane = ActivePaneStyle.Width(rightWidth - 2).Height(m.height - 3).Render(m.viewport.View())
	}

	content := lipgloss.JoinHorizontal(lipgloss.Top, leftPane, rightPane)

	var statusBar string
	if m.running {
		statusBar = StatusRunningStyle.Render(fmt.Sprintf("Running %s on %s...", m.activeCmd, m.activeHost))
	} else {
		statusBar = m.help.View(m.keys)
	}

	return lipgloss.JoinVertical(lipgloss.Left, content, statusBar)
}
