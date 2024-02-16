extends LineEdit
@onready var CLOCKER: ClockerConnection = %Clocker
var message_box = "Comment"
var name_box = "?"
var my_comment = false

func _ready():
	await CLOCKER.ready
	CLOCKER.connect(CLOCKER.signal_new_textchat_message(), _on_new_textchat_message)

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_ENTER:
			select()
			CLOCKER.oneshot_send_chat_message(get_selected_text())
			clear()
			my_comment = true

func _on_new_textchat_message(sender, message):
	name_box = sender
	message_box = message
	print("プレイヤー%s 「%s」" % [sender, message])
	pass # Replace with function body.
