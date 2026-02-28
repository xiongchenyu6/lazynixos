package main

import (
	"flag"
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
)

var Program *tea.Program

func main() {
	var flakeFlag string
	flag.StringVar(&flakeFlag, "flake", "", "Path to flake directory")
	flag.Parse()

	flakePath := flakeFlag
	if flakePath == "" {
		flakePath = os.Getenv("LAZYNIXOS_FLAKE")
	}
	if flakePath == "" {
		flakePath = "/home/freeman.xiong/dotfiles"
	}

	flakeInfo, err := os.Stat(flakePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: flake path %q not found: %v\n", flakePath, err)
		os.Exit(1)
	}
	if !flakeInfo.IsDir() {
		fmt.Fprintf(os.Stderr, "Error: flake path %q is not a directory\n", flakePath)
		os.Exit(1)
	}

	m := NewModel(flakePath)
	p := tea.NewProgram(m, tea.WithAltScreen(), tea.WithMouseCellMotion())
	Program = p
	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
