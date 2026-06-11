package export_wit_world

import (
	"fmt"
	"math"
	"strconv"
	"wit_component/local_zappy_command"
	"wit_component/local_zappy_graphic"
	"wit_component/local_zappy_host_api"
	"wit_component/local_zappy_input"
	"wit_component/local_zappy_system"
)

var initialized = false
var flashEffect float32 = 0.0

var commandsBuf []local_zappy_graphic.RenderCommand

var timeScale float32 = 1.0

func initModule() {
	if initialized {
		return
	}

	local_zappy_host_api.HostSubscribe("cube:bounce")
	local_zappy_host_api.HostSubscribe("env:tick")
	local_zappy_host_api.HostSubscribe("sys:timescale_changed")

	local_zappy_host_api.HostSetState("bg:current", []byte("Synthwave Grid"))
	local_zappy_host_api.HostLog("Go Module: Background Manager initialized")

	commandsBuf = make([]local_zappy_graphic.RenderCommand, 0, 64)

	initialized = true
}

func UpdateModule(time float32, dt float32, w float32, h float32) []local_zappy_graphic.RenderCommand {
	initModule()

	commands := commandsBuf[:0]

	bgCmd := local_zappy_graphic.MakeRenderCommandRect(local_zappy_graphic.RectCmd{
		X: 0, Y: 0, W: w, H: h,
		Color:    local_zappy_graphic.Color{R: 20, G: 20, B: 30, A: 255},
		Rotation: 0,
	})
	commands = append(commands, bgCmd)

	if flashEffect > 0 {
		flashCmd := local_zappy_graphic.MakeRenderCommandRect(local_zappy_graphic.RectCmd{
			X: 0, Y: 0, W: w, H: h,
			Color:    local_zappy_graphic.Color{R: 255, G: 255, B: 255, A: uint8(flashEffect * 100)},
			Rotation: 0,
		})
		commands = append(commands, flashCmd)
		flashEffect -= dt * 2.0
	}

	offset := float32(math.Mod(float64(time*50.0*timeScale), 100.0))
	for y := float32(0); y < h; y += 100 {
		line := local_zappy_graphic.MakeRenderCommandRect(local_zappy_graphic.RectCmd{
			X: 0, Y: y + offset, W: w, H: 2,
			Color:    local_zappy_graphic.Color{R: 100, G: 50, B: 200, A: 100},
			Rotation: 0,
		})
		commands = append(commands, line)
	}

	commandsBuf = commands

	return commands
}

func HandleEvent(eventName string, payload string) {
	if eventName == "cube:bounce" {
		flashEffect = 1.0
		local_zappy_host_api.HostLog("The background saw the cube bouncing!")
	}

	if eventName == "sys:timescale_changed" {
		if val, err := strconv.ParseFloat(payload, 32); err == nil {
			timeScale = float32(val)
		}
	}
}

func RunCommand(cmd string, args []string) local_zappy_command.ResponseCommand {
	if cmd == "bg" && len(args) > 0 {
		local_zappy_host_api.HostSetState("bg:current", []byte(args[0]))
		local_zappy_host_api.HostLog(fmt.Sprintf("Background changed to: %s", args[0]))
		return local_zappy_command.MakeResponseCommandOk()
	}
	return local_zappy_command.MakeResponseCommandUnknown()
}

func HandleInput(state local_zappy_input.InputState) {}
func GetCommands() []local_zappy_command.CommandDesc {
	return []local_zappy_command.CommandDesc{
		{Module: "bg_manager", Name: "bg", Options: "<name>", Help: "Change the background style"},
	}
}
func AcceptLog(segments []local_zappy_system.TextSegment) {}
func Serialize() []uint8                                  { return []uint8{} }
func Deserialize(state []uint8)                           {}
