extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	var clocker = get_parent()
	await(clocker.ready)
	clocker.connect_to_localhost()
	clocker.join_instance(1)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
