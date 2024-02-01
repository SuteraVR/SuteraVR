extends LineEdit

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_ENTER:
			select()
			print(get_selected_text())
			clear()
