extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	var clocker_connection = get_parent()
	await(clocker_connection.ready)
	clocker_connection.join_instance(1)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
