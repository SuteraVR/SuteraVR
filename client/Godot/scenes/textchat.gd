extends LineEdit
var CLOCKER: ClockerConnection
func _ready():
	CLOCKER = %Clocker

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_ENTER:
			select()
			CLOCKER.oneshot_send_chat_message(get_selected_text())
			clear()
