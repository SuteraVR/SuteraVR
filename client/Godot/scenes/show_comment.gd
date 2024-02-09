extends Label

@export var line_edit:Control

func _process(delta):
	text = line_edit.message_box
