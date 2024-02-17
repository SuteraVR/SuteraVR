extends Label

@onready var line_edit: LineEdit = %LineEdit

func _process(delta):
	text = line_edit.message_box
