extends Label

@export var line_edit:Control

func _process(delta):
	text = "%s" % line_edit.message_box
