extends Label

@export var line_edit:Control
@onready var instance_anchor:Node = %InstanceAnchor

func _process(delta):
	var comment = line_edit.message_box
	if (!line_edit.my_comment):
		if(comment.left(2) == "//"):
			var comment_array = comment.rsplit(" ",true)
			if(comment_array[0]=="//ch_av"):
				instance_anchor.change_user_avatar(line_edit.name_box,comment_array[1].to_int())
				text = "changed avatar number %s" % comment_array[1]
			else:
				text = "invalid command"
		else:
			text = "%s" % comment
	else:
		text = "%s" % comment
	
