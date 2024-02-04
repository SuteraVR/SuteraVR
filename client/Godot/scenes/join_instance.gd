extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	var clocker = get_parent()
	
	# ClockerがReadyになるのを待機
	await(clocker.ready)
	
	# ホストに接続し、通信確立を待機
	# 
	# 例) ローカルでclocking-serverを動かしている場合:
	#   clocker.connect_to_localhost()
	#
	# 例) 外部のサーバーに接続する場合
	#   clocker.connect_by_srv("suteravr.io")
	await Signal(clocker, clocker.signal_connection_established())
	
	# インスタンスに参加
	clocker.join_instance(1)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
