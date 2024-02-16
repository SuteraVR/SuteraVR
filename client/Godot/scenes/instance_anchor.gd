extends Node

@onready var clocker: ClockerConnection = get_parent()
var player_scene = preload("res://scenes/instance_player.tscn")
var player_instances = {}

# Called when the node enters the scene tree for the first time.
func _ready():
	await clocker.ready
	clocker.connect(clocker.signal_update_player_being(), _on_update_player_being)
	clocker.connect(clocker.signal_player_moved(), _on_player_moved)
	
	# ホストに接続し、通信確立を待機
	# 
	# 例) ローカルでclocking-serverを動かしている場合:
	#    clocker.connect_to_localhost()
	#
	# 例) 外部のサーバーに接続する場合
	clocker.connect_by_srv("aspulse-host.suteravr.io")
	
	await Signal(clocker, clocker.signal_connection_established())
	
	# インスタンスに参加
	clocker.join_instance(1)	


func _on_update_player_being(id: int, value: bool, joining: bool):
	if value == true:
		push_player(id)
		if !joining:
			print('プレイヤー%sが参加しました' % [id])
	if value == false:
		delete_player_instance(id)
		if !joining:
			print('プレイヤー%sが離脱しました' % [id])

func _on_player_moved(
	id: int,
	x: float, y: float, z: float,
	xx: float, xz: float, zx: float, zz: float,
):
	var player = get_player(id)
	if player == null:
		return
	player.appear()
	player.move(x, y, z, xx, xz, zx, zz)

func push_player(id: int) -> PlayerInstance:
	var instance = PlayerInstance.new(clocker, id)
	player_instances[id] = instance
	add_child(instance)
	return instance

func get_player(id: int) -> PlayerInstance:
	if !player_instances.has(id):
		return null
	return player_instances[id]

func delete_player_instance(id: int):
	var instance = get_player(id)
	if instance != null && is_instance_valid(instance):
		instance.free()
		player_instances[id] = null
		
func change_user_avatar(id:int,avatar_num:int):
	var player = get_player(id)
	player.change_avatar(avatar_num)
