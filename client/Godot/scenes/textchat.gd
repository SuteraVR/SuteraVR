extends LineEdit
@onready var CLOCKER: ClockerConnection = %Clocker
@onready var instance_anchor:Node = %InstanceAnchor
@onready var player_body:CharacterBody3D = %CharacterBody3D
var message_box = "Comment"
var name_box = "?"

func _ready():
	await CLOCKER.ready
	CLOCKER.connect(CLOCKER.signal_new_textchat_message(), _on_new_textchat_message)

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_ENTER:
			select()
			CLOCKER.oneshot_send_chat_message(get_selected_text())
			clear()

func _on_new_textchat_message(sender, message):
	name_box = sender
	message_box = message
	print("プレイヤー%s 「%s」" % [sender, message])
	if(message.left(2) == "//"):
		var comment_array = message.rsplit(" ",true)
		if(comment_array[0]=="//ch_av"):
			message_box = "changed avatar number %s" % comment_array[1]
			instance_anchor.change_user_avatar(sender, comment_array[1].to_int())
		elif(comment_array[0] == "//ch_world"):
			teleport_world(comment_array[1])
		else:
			message_box = "invalid command"
		return

func teleport_world(world_num: String):
	if(world_num=="school"):
		player_body.accept_teleport(Vector3(0,0,0))
		print("teleporting")
	if(world_num=="nekoyama"):
		player_body.accept_teleport(Vector3(100,200,0))
		print("teleporting")
	if(world_num=="museum2"):
		player_body.accept_teleport(Vector3(300,100,300))
		print("teleporting")
