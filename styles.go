package main

import "github.com/charmbracelet/lipgloss"

const LeftPaneRatio = 0.25

var (
	// Subtle color palette
	subtle    = lipgloss.AdaptiveColor{Light: "#D9DCCF", Dark: "#383838"}
	highlight = lipgloss.AdaptiveColor{Light: "#874BFD", Dark: "#7D56F4"}
	special   = lipgloss.AdaptiveColor{Light: "#43BF6D", Dark: "#73F59F"}
	errorClr  = lipgloss.AdaptiveColor{Light: "#FF0000", Dark: "#FF6666"}

	// Left pane (host list) - with rounded border
	ActivePaneStyle = lipgloss.NewStyle().
			BorderStyle(lipgloss.RoundedBorder()).
			BorderForeground(highlight).
			Padding(0, 1)

	InactivePaneStyle = lipgloss.NewStyle().
				BorderStyle(lipgloss.RoundedBorder()).
				BorderForeground(subtle).
				Padding(0, 1)

	// Status bar at the bottom
	StatusBarStyle = lipgloss.NewStyle().
			Foreground(lipgloss.AdaptiveColor{Light: "#FFFDF5", Dark: "#FFFDF5"}).
			Background(lipgloss.AdaptiveColor{Light: "#6124DF", Dark: "#7D56F4"}).
			Padding(0, 1)

	// Status text variants
	StatusRunningStyle = lipgloss.NewStyle().
				Foreground(lipgloss.AdaptiveColor{Light: "#FFFDF5", Dark: "#FFFDF5"}).
				Background(lipgloss.AdaptiveColor{Light: "#F25D94", Dark: "#F25D94"}).
				Padding(0, 1).
				Bold(true)

	StatusSuccessStyle = lipgloss.NewStyle().
				Foreground(lipgloss.AdaptiveColor{Light: "#FFFDF5", Dark: "#FFFDF5"}).
				Background(lipgloss.AdaptiveColor{Light: "#43BF6D", Dark: "#73F59F"}).
				Padding(0, 1).
				Bold(true)

	// Error line in logs
	ErrorLineStyle = lipgloss.NewStyle().
			Foreground(errorClr)

	// Title style for the list
	TitleStyle = lipgloss.NewStyle().
			Foreground(highlight).
			Bold(true).
			Padding(0, 1)
)
