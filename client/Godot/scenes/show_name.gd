extends Label

@export var line_edit: LineEdit

func _process(delta):
	text = "Player%s" % line_edit.name_box
