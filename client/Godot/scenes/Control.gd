extends CanvasItem
var state

# Called when the node enters the scene tree for the first time.
func _ready():
	visible = false
	state = false


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_TAB:
			if !state:
				visible = true
				state = true
			else:
				state = false
				visible = false
