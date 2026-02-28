package main

import (
	"encoding/json"
	"os/exec"
	"sort"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type HostItem struct {
	name string
}

var _ list.Item = HostItem{}

func (h HostItem) Title() string {
	return h.name
}

func (h HostItem) Description() string {
	return ""
}

func (h HostItem) FilterValue() string {
	return h.name
}

type HostsLoadedMsg struct {
	hosts []HostItem
}

type HostsErrorMsg struct {
	err error
}

func FetchHosts(flakePath string) tea.Cmd {
	return func() tea.Msg {
		cmd := exec.Command(
			"nix",
			"eval",
			"--json",
			".#nixosConfigurations",
			"--apply",
			"builtins.attrNames",
		)
		cmd.Dir = flakePath

		output, err := cmd.Output()
		if err != nil {
			return HostsErrorMsg{err: err}
		}

		var hostNames []string
		if err := json.Unmarshal(output, &hostNames); err != nil {
			return HostsErrorMsg{err: err}
		}

		sort.Strings(hostNames)

		hosts := make([]HostItem, len(hostNames))
		for i, hostName := range hostNames {
			hosts[i] = HostItem{name: hostName}
		}

		return HostsLoadedMsg{hosts: hosts}
	}
}
