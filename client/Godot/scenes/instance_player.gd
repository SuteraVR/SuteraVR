extends Node
var CLOCKER: ClockerConnection

func _ready():
	CLOCKER = %Clocker
	await CLOCKER.ready
	CLOCKER.connect(CLOCKER.signal_update_player_being(), _on_update_player_being)

func _on_update_player_being(id: int, value: bool):
	if value == true:
		print('プレイヤー%sが参加しました' % [id])
	if value == false:
		print('プレイヤー%sが離脱しました' % [id])
