extends LineEdit
@onready var CLOCKER: ClockerConnection = %Clocker
@onready var instance_anchor:Node = %InstanceAnchor
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
		else:
			message_box = "invalid command"
		return
