extends Label

@export var line_edit:Control
@onready var instance_anchor:Node = %InstanceAnchor
@onready var player_body:CharacterBody3D = %CharacterBody3D

func _process(delta):
	var comment = line_edit.message_box
	if (!line_edit.my_comment):
		if(comment.left(2) == "//"):
			var comment_array = comment.rsplit(" ",true)
			if(comment_array[0]=="//ch_av"):
				instance_anchor.change_user_avatar(line_edit.name_box,comment_array[1].to_int())
				text = "changed avatar number %s" % comment_array[1]
			elif(comment_array[0] == "//ch_world"):
				teleport_world(comment_array[1].to_int())
			else:
				text = "invalid command"
		else:
			text = "%s" % comment
	else:
		text = "%s" % comment
	line_edit.my_comment = false

func teleport_world(world_num:int):
	const world1 = Vector3(3,0,0)
	if(world_num==1):
		player_body.accept_teleport(world1)
		print("teleporting")
