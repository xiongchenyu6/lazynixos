package main

import (
	"bufio"
	"errors"
	"io"
	"os/exec"
	"sync"

	tea "github.com/charmbracelet/bubbletea"
)

type CommandStartedMsg struct {
	Host   string
	Action string // "switch", "build", "dry-build"
}

type LogLineMsg struct {
	Line     string
	IsStderr bool
}

type CommandFinishedMsg struct {
	Host   string
	Action string
	Err    error
}

func RunNixosRebuild(flakePath, host, action string) tea.Cmd {
	return func() tea.Msg {
		if Program == nil {
			return LogLineMsg{Line: "Cannot run command: program not initialized", IsStderr: true}
		}

		go runNixosRebuild(flakePath, host, action)
		return nil
	}
}

func runNixosRebuild(flakePath, host, action string) {
	sendToProgram(CommandStartedMsg{Host: host, Action: action})

	args := []string{
		action,
		"--flake", flakePath + "#" + host,
		"--use-substitutes",
		"--target-host", "root@" + host,
		"--impure",
	}

	cmd := exec.Command("nixos-rebuild", args...)

	stdoutPipe, err := cmd.StdoutPipe()
	if err != nil {
		sendToProgram(CommandFinishedMsg{Host: host, Action: action, Err: err})
		return
	}

	stderrPipe, err := cmd.StderrPipe()
	if err != nil {
		sendToProgram(CommandFinishedMsg{Host: host, Action: action, Err: err})
		return
	}

	if err := cmd.Start(); err != nil {
		sendToProgram(CommandFinishedMsg{Host: host, Action: action, Err: err})
		return
	}

	var wg sync.WaitGroup
	var scanErr error
	var scanErrMu sync.Mutex

	scanStream := func(stream io.ReadCloser, isStderr bool) {
		defer wg.Done()

		scanner := bufio.NewScanner(stream)
		for scanner.Scan() {
			sendToProgram(LogLineMsg{Line: scanner.Text(), IsStderr: isStderr})
		}

		if err := scanner.Err(); err != nil {
			scanErrMu.Lock()
			if scanErr == nil {
				scanErr = err
			}
			scanErrMu.Unlock()
		}
	}

	wg.Add(2)
	go scanStream(stdoutPipe, false)
	go scanStream(stderrPipe, true)
	wg.Wait()

	err = cmd.Wait()
	if scanErr != nil {
		err = errors.Join(err, scanErr)
	}

	sendToProgram(CommandFinishedMsg{Host: host, Action: action, Err: err})
}

func sendToProgram(msg tea.Msg) {
	if Program == nil {
		return
	}

	Program.Send(msg)
}
